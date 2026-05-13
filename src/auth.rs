use anyhow::{bail, Result};
use base64::Engine;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

// ── .claude/auth.json 存储 ──────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct AuthStore {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_token: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub id_token: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_expiry: Option<u64>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gateway: Option<serde_json::Value>,
}

impl AuthStore {
    /// 从 `.claude/auth.json` 读取
    pub fn load(root: &PathBuf) -> Result<Self> {
        let path = root.join(".claude").join("auth.json");
        if !path.exists() {
            return Ok(AuthStore::default());
        }
        let data = std::fs::read_to_string(&path)?;
        Ok(serde_json::from_str(&data)?)
    }

    /// 从 OAuth token 响应构建（不含 gateway，合并写入时会保留已有 gateway）
    pub fn from_tokens(
        access_token: String,
        refresh_token: Option<String>,
        id_token: Option<String>,
        token_expiry: Option<u64>,
    ) -> Self {
        AuthStore {
            access_token: Some(access_token),
            refresh_token,
            id_token,
            token_expiry,
            gateway: None,
        }
    }

    /// 将 token 字段合并写入 auth.json。
    /// 只更新 access_token / refresh_token / id_token / token_expiry 四个字段，
    /// gateway 及其他已有配置原样保留，绝不清除用户数据。
    pub fn merge_tokens(&self, root: &PathBuf) {
        let dir = root.join(".claude");
        std::fs::create_dir_all(&dir).ok();
        let path = dir.join("auth.json");

        let mut json: Value = if path.exists() {
            let data = std::fs::read_to_string(&path).unwrap_or_default();
            serde_json::from_str(&data).unwrap_or(Value::Object(serde_json::Map::new()))
        } else {
            Value::Object(serde_json::Map::new())
        };

        if let Some(ref t) = self.access_token {
            json["access_token"] = Value::String(t.clone());
        }
        if let Some(ref t) = self.refresh_token {
            json["refresh_token"] = Value::String(t.clone());
        }
        if let Some(ref t) = self.id_token {
            json["id_token"] = Value::String(t.clone());
        }
        if let Some(exp) = self.token_expiry {
            json["token_expiry"] = Value::Number(exp.into());
        }

        std::fs::write(&path, serde_json::to_string_pretty(&json).unwrap()).ok();
    }

    /// 检查 access_token 是否存在且未过期
    pub fn is_token_valid(&self) -> bool {
        let token = match &self.access_token {
            Some(t) if !t.is_empty() => t,
            _ => return false,
        };
        if let Ok(payload) = decode_jwt_payload(token) {
            if payload.exp > 0 {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                return payload.exp > now;
            }
        }
        true
    }

    /// 从 id_token 解析用户信息
    pub fn parse_user(&self) -> Option<UserInfo> {
        let id_token = self.id_token.as_ref()?;
        let payload = decode_jwt_payload(id_token).ok()?;

        let phone = if !payload.phone_number.is_empty() {
            mask_phone(&payload.phone_number)
        } else {
            String::from("未知")
        };

        Some(UserInfo {
            phone,
            balance: "¥ 50.00".to_string(),
        })
    }
}

// ── 用户信息（GUI 展示用）──────────────────────────────────

#[derive(Debug, Clone, Default)]
pub struct UserInfo {
    pub phone: String,
    pub balance: String,
}

// ── 状态机 ──────────────────────────────────────────────────

pub type LoginResult = Result<UserInfo, String>;

pub enum LoginState {
    LoggedOut,
    LoggingIn(Arc<Mutex<Option<LoginResult>>>),
    LoggedIn(UserInfo),
    Launching,
}


// ── JWT Payload ─────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct JwtPayload {
    #[serde(default)]
    exp: u64,

    #[serde(default)]
    phone_number: String,
}

fn mask_phone(phone: &str) -> String {
    let chars: Vec<char> = phone.chars().collect();
    if chars.len() < 7 {
        return phone.to_string();
    }
    format!(
        "{} **** {}",
        chars[..3].iter().collect::<String>(),
        chars[chars.len() - 4..].iter().collect::<String>(),
    )
}

fn decode_jwt_payload(token: &str) -> Result<JwtPayload> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() < 2 {
        bail!("invalid JWT: not enough parts");
    }
    let payload_json = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(parts[1])?;
    Ok(serde_json::from_slice(&payload_json)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn merge_tokens_preserves_gateway_and_other_fields() {
        let dir = std::env::temp_dir().join("dgpt_merge_test");
        let _ = fs::remove_dir_all(&dir);
        let claude = dir.join(".claude");
        fs::create_dir_all(&claude).unwrap();

        // 模拟用户已有的 auth.json —— 包含 gateway 和自定义字段
        let original = serde_json::json!({
            "access_token": "old_token",
            "refresh_token": "old_refresh",
            "id_token": "old_id",
            "token_expiry": 1,
            "gateway": {
                "OPENAI_API_KEY": "sk-keep-me",
                "OPENAI_BASE_URL": "https://my.url/v1",
                "OPENAI_IMAGE_MODEL": "gpt-image-2"
            },
            "custom_field": "should_survive"
        });
        fs::write(
            claude.join("auth.json"),
            serde_json::to_string_pretty(&original).unwrap(),
        )
        .unwrap();

        // OAuth 回执：新 token
        let new_auth = AuthStore::from_tokens(
            "new_access".into(),
            Some("new_refresh".into()),
            Some("new_id".into()),
            Some(9999999999u64),
        );
        new_auth.merge_tokens(&dir);

        // 读取结果
        let result: Value =
            serde_json::from_str(&fs::read_to_string(claude.join("auth.json")).unwrap())
                .unwrap();

        // Token 字段应该更新
        assert_eq!(result["access_token"], "new_access");
        assert_eq!(result["refresh_token"], "new_refresh");
        assert_eq!(result["id_token"], "new_id");
        assert_eq!(result["token_expiry"], 9999999999u64);

        // Gateway 应该原样保留
        assert_eq!(result["gateway"]["OPENAI_API_KEY"], "sk-keep-me");
        assert_eq!(result["gateway"]["OPENAI_BASE_URL"], "https://my.url/v1");
        assert_eq!(result["gateway"]["OPENAI_IMAGE_MODEL"], "gpt-image-2");

        // 其他自定义字段也不该丢
        assert_eq!(result["custom_field"], "should_survive");

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn merge_tokens_creates_new_file_if_missing() {
        let dir = std::env::temp_dir().join("dgpt_merge_new");
        let _ = fs::remove_dir_all(&dir);

        let auth = AuthStore::from_tokens(
            "at".into(),
            Some("rt".into()),
            Some("idt".into()),
            Some(100u64),
        );
        auth.merge_tokens(&dir);

        let result: Value =
            serde_json::from_str(
                &fs::read_to_string(dir.join(".claude").join("auth.json")).unwrap(),
            )
            .unwrap();

        assert_eq!(result["access_token"], "at");
        assert_eq!(result["refresh_token"], "rt");
        assert_eq!(result["id_token"], "idt");
        assert_eq!(result["token_expiry"], 100u64);
        // 新文件不应有 gateway
        assert!(result.get("gateway").is_none());

        fs::remove_dir_all(&dir).ok();
    }
}
