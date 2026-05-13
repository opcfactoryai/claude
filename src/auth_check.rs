use crate::auth::{AuthStore, UserInfo};
use std::path::PathBuf;

/// 启动时唯一的登录状态检查入口。
///
/// 校验 auth.json 结构完整性 + JWT 过期时间：
///   - access_token / refresh_token / id_token 非空
///   - token_expiry 存在
///   - gateway 节点存在
///   - JWT 未过期
///
/// 五项缺一不可。全部通过返回 Some(UserInfo)，否则返回 None（GUI 显示登录按钮）。
/// 此函数只读，不写入任何文件。
pub fn check_login(root: &PathBuf) -> Option<UserInfo> {
    let auth = AuthStore::load(root).ok()?;

    if auth.access_token.as_ref()?.is_empty() {
        return None;
    }
    if auth.refresh_token.as_ref()?.is_empty() {
        return None;
    }
    if auth.id_token.as_ref()?.is_empty() {
        return None;
    }
    auth.token_expiry?;
    auth.gateway.as_ref()?;

    if !auth.is_token_valid() {
        return None;
    }

    auth.parse_user()
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine;
    use std::fs;

    /// 构造一个最小可用的 JWT token（仅测试用，签名伪造）
    fn make_jwt(exp: u64, phone: &str) -> String {
        let header = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .encode(b"{\"alg\":\"HS256\",\"typ\":\"JWT\"}");
        let payload = format!(
            r#"{{"sub":"test","phone_number":"{}","exp":{}}}"#,
            phone, exp
        );
        let body = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .encode(payload.as_bytes());
        let sig = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .encode(b"fake");
        format!("{}.{}.{}", header, body, sig)
    }

    fn future_exp() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 3600
    }

    fn past_exp() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            - 3600
    }

    fn write_auth_json(dir: &PathBuf, json: &serde_json::Value) {
        let claude = dir.join(".claude");
        fs::create_dir_all(&claude).unwrap();
        fs::write(
            claude.join("auth.json"),
            serde_json::to_string_pretty(json).unwrap(),
        )
        .unwrap();
    }

    fn valid_auth_json(exp: u64) -> serde_json::Value {
        let token = make_jwt(exp, "13800000000");
        serde_json::json!({
            "access_token": token,
            "refresh_token": token,
            "id_token": token,
            "token_expiry": exp,
            "gateway": {
                "ENABLE_GARDEN_IMAGEGEN": "1",
                "OPENAI_API_KEY": "sk-test",
                "OPENAI_BASE_URL": "https://test/v1"
            }
        })
    }

    #[test]
    fn all_valid_returns_some() {
        let dir = std::env::temp_dir().join("dgpt_test_valid");
        let _ = fs::remove_dir_all(&dir);
        write_auth_json(&dir, &valid_auth_json(future_exp()));
        let result = check_login(&dir);
        fs::remove_dir_all(&dir).ok();
        assert!(result.is_some(), "valid auth.json should return Some");
    }

    #[test]
    fn expired_jwt_returns_none() {
        let dir = std::env::temp_dir().join("dgpt_test_expired");
        let _ = fs::remove_dir_all(&dir);
        write_auth_json(&dir, &valid_auth_json(past_exp()));
        let result = check_login(&dir);
        fs::remove_dir_all(&dir).ok();
        assert!(result.is_none(), "expired JWT should return None");
    }

    #[test]
    fn missing_access_token_returns_none() {
        let dir = std::env::temp_dir().join("dgpt_test_no_at");
        let _ = fs::remove_dir_all(&dir);
        let mut json = valid_auth_json(future_exp());
        json["access_token"] = serde_json::Value::Null;
        write_auth_json(&dir, &json);
        let result = check_login(&dir);
        fs::remove_dir_all(&dir).ok();
        assert!(result.is_none());
    }

    #[test]
    fn missing_gateway_returns_none() {
        let dir = std::env::temp_dir().join("dgpt_test_no_gw");
        let _ = fs::remove_dir_all(&dir);
        let mut json = valid_auth_json(future_exp());
        json.as_object_mut().unwrap().remove("gateway");
        write_auth_json(&dir, &json);
        let result = check_login(&dir);
        fs::remove_dir_all(&dir).ok();
        assert!(result.is_none());
    }

    #[test]
    fn missing_refresh_token_returns_none() {
        let dir = std::env::temp_dir().join("dgpt_test_no_rt");
        let _ = fs::remove_dir_all(&dir);
        let mut json = valid_auth_json(future_exp());
        json["refresh_token"] = serde_json::Value::Null;
        write_auth_json(&dir, &json);
        let result = check_login(&dir);
        fs::remove_dir_all(&dir).ok();
        assert!(result.is_none());
    }

    #[test]
    fn missing_id_token_returns_none() {
        let dir = std::env::temp_dir().join("dgpt_test_no_id");
        let _ = fs::remove_dir_all(&dir);
        let mut json = valid_auth_json(future_exp());
        json["id_token"] = serde_json::Value::Null;
        write_auth_json(&dir, &json);
        let result = check_login(&dir);
        fs::remove_dir_all(&dir).ok();
        assert!(result.is_none());
    }

    #[test]
    fn missing_token_expiry_returns_none() {
        let dir = std::env::temp_dir().join("dgpt_test_no_exp");
        let _ = fs::remove_dir_all(&dir);
        let mut json = valid_auth_json(future_exp());
        json.as_object_mut().unwrap().remove("token_expiry");
        write_auth_json(&dir, &json);
        let result = check_login(&dir);
        fs::remove_dir_all(&dir).ok();
        assert!(result.is_none());
    }

    #[test]
    fn empty_access_token_returns_none() {
        let dir = std::env::temp_dir().join("dgpt_test_empty_at");
        let _ = fs::remove_dir_all(&dir);
        let mut json = valid_auth_json(future_exp());
        json["access_token"] = serde_json::Value::String(String::new());
        write_auth_json(&dir, &json);
        let result = check_login(&dir);
        fs::remove_dir_all(&dir).ok();
        assert!(result.is_none());
    }

    #[test]
    fn no_auth_json_file_returns_none() {
        let dir = std::env::temp_dir().join("dgpt_test_no_file");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let result = check_login(&dir);
        fs::remove_dir_all(&dir).ok();
        assert!(result.is_none());
    }
}
