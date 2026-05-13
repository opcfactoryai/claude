use anyhow::{bail, Context, Result};
use base64::Engine;
use rand::Rng;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::time::Duration;

use crate::auth::{AuthStore, UserInfo};

// ── OAuth 2.0 配置 ───────────────────────────────────────────

pub struct OAuthConfig {
    pub authorize_url: String,
    pub token_url: String,
    pub client_id: String,
    pub scopes: String,
}

impl Default for OAuthConfig {
    fn default() -> Self {
        Self {
            authorize_url: std::env::var("OAUTH_AUTHORIZE_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:3000/authorize".to_string()),
            token_url: std::env::var("OAUTH_TOKEN_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:3000/api/auth/token".to_string()),
            client_id: std::env::var("OAUTH_CLIENT_ID")
                .unwrap_or_else(|_| "designgpt-desktop".to_string()),
            scopes: std::env::var("OAUTH_SCOPES")
                .unwrap_or_else(|_| "openid profile".to_string()),
        }
    }
}

// ── Token 交换响应 ───────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: Option<String>,
    id_token: Option<String>,
    refresh_token: Option<String>,
    expires_in: Option<u64>,
    #[serde(default)]
    error: String,
    #[serde(default)]
    error_description: String,
}

// ── PKCE 工具 ────────────────────────────────────────────────

fn generate_code_verifier() -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~";
    let mut rng = rand::thread_rng();
    let len = rng.gen_range(43..=128);
    (0..len)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

fn generate_code_challenge(verifier: &str) -> String {
    let hash = Sha256::digest(verifier.as_bytes());
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(hash)
}

fn generate_state() -> String {
    let mut rng = rand::thread_rng();
    let len = rng.gen_range(16..=32);
    (0..len)
        .map(|_| {
            let idx = rng.gen_range(0..62u8);
            match idx {
                0..=9 => (b'0' + idx) as char,
                10..=35 => (b'A' + (idx - 10)) as char,
                _ => (b'a' + (idx - 36)) as char,
            }
        })
        .collect()
}

// ── HTTP 回调 ────────────────────────────────────────────────

const SUCCESS_HTML: &str = "\
<!DOCTYPE html>
<html>
<head><meta charset=\"utf-8\"><title>DesignGPT - 登录成功</title></head>
<body style=\"font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif; text-align: center; padding-top: 80px; background: #1a1a2e; color: #e0e0e0;\">
  <h1 style=\"color: #4fc3f7;\">登录成功</h1>
  <p>正在返回 DesignGPT 应用，此页面可以关闭。</p>
</body>
</html>";

fn http_ok(body: &str) -> String {
    format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    )
}

/// 阻塞等待浏览器回调，解析 code / state
fn wait_for_callback(listener: TcpListener) -> Result<(String, String)> {
    let (mut stream, _) = listener.accept().context("TCP accept failed")?;
    let request = read_request(&mut stream)?;

    // 解析首行: GET /cb?code=XXX&state=YYY HTTP/1.1
    let first_line = request.lines().next().unwrap_or("");
    let path = first_line
        .split_whitespace()
        .nth(1)
        .unwrap_or("");

    if !path.starts_with("/cb?") {
        // 不是回调请求（可能是 favicon 等），返回 404 并继续等待
        let _ = stream.write_all(b"HTTP/1.1 404 Not Found\r\n\r\n");
        drop(stream);
        return wait_for_callback(listener);
    }

    let query = path.splitn(2, '?').nth(1).unwrap_or("");
    let code = extract_param(query, "code").context("missing code in callback")?;
    let state = extract_param(query, "state").context("missing state in callback")?;

    // 返回成功页面
    let _ = stream.write_all(http_ok(SUCCESS_HTML).as_bytes());

    Ok((code, state))
}

fn read_request(stream: &mut TcpStream) -> Result<String> {
    let mut buf = [0u8; 4096];
    let n = stream.read(&mut buf).context("failed to read HTTP request")?;
    Ok(String::from_utf8_lossy(&buf[..n]).to_string())
}

fn extract_param(query: &str, key: &str) -> Option<String> {
    let prefix = format!("{}=", key);
    for pair in query.split('&') {
        if let Some(rest) = pair.strip_prefix(&prefix) {
            return Some(url_decode(rest));
        }
    }
    // key 可能在末尾，值在查询字符串末尾
    None
}

// ── 主流程 ───────────────────────────────────────────────────

/// 执行完整的 OAuth 2.0 PKCE 登录流程。
/// 获取 token 后调用 AuthStore::merge_tokens 持久化（只写 4 个 token 字段），
/// 返回用户信息供 GUI 展示。
pub fn login(root: &PathBuf) -> Result<UserInfo> {
    let config = OAuthConfig::default();

    // 1. 生成 PKCE 参数
    let code_verifier = generate_code_verifier();
    let code_challenge = generate_code_challenge(&code_verifier);
    let state = generate_state();

    // 2. 启动回调监听（端口固定，redirect_uri 可预注册）
    const CALLBACK_PORT: u16 = 8765;
    let redirect_uri = format!("http://127.0.0.1:{}/cb", CALLBACK_PORT);
    let listener = TcpListener::bind(format!("127.0.0.1:{}", CALLBACK_PORT))?;

    // 3. 构建授权 URL 并打开浏览器
    let auth_url = format!(
        "{}?response_type=code&client_id={}&redirect_uri={}&code_challenge={}&\
         code_challenge_method=S256&state={}&scope={}",
        config.authorize_url,
        url_encode(&config.client_id),
        url_encode(&redirect_uri),
        url_encode(&code_challenge),
        url_encode(&state),
        url_encode(&config.scopes),
    );

    open_browser(&auth_url)?;

    // 4. 阻塞等待浏览器回调
    let (code, returned_state) = wait_for_callback(listener)?;

    // 5. 验证 state
    if returned_state != state {
        bail!(
            "state mismatch: expected {}, got {}",
            state,
            returned_state
        );
    }

    // 6. 用 code + verifier 换 token
    let token_body = format!(
        "grant_type=authorization_code&code={}&redirect_uri={}&\
         client_id={}&code_verifier={}",
        url_encode(&code),
        url_encode(&redirect_uri),
        url_encode(&config.client_id),
        url_encode(&code_verifier),
    );

    let token_resp: TokenResponse = ureq::post(&config.token_url)
        .set("Content-Type", "application/x-www-form-urlencoded")
        .set("Accept", "application/json")
        .timeout(Duration::from_secs(30))
        .send_string(&token_body)
        .context("failed to exchange token")?
        .into_json()
        .context("failed to parse token response")?;

    if !token_resp.error.is_empty() {
        bail!(
            "token error: {} - {}",
            token_resp.error,
            token_resp.error_description
        );
    }

    let access_token = token_resp.access_token.context("missing access_token")?;

    // 7. 构建 AuthStore 并持久化（只写 4 个 token 字段，不动 gateway）
    let token_expiry = token_resp.expires_in.map(|s| {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + s
    });

    let auth = AuthStore::from_tokens(
        access_token,
        token_resp.refresh_token,
        token_resp.id_token,
        token_expiry,
    );

    let user = auth.parse_user().unwrap_or_default();
    auth.merge_tokens(root);

    Ok(user)
}

// ── 打开浏览器 (ShellExecuteW) ───────────────────────────────

#[link(name = "shell32")]
extern "system" {
    fn ShellExecuteW(
        hwnd: isize,
        lpOperation: *const u16,
        lpFile: *const u16,
        lpParameters: *const u16,
        lpDirectory: *const u16,
        nShowCmd: i32,
    ) -> isize;
}

const SW_SHOW: i32 = 5;

fn open_browser(url: &str) -> Result<()> {
    let op = to_wide("open");
    let file = to_wide(url);

    let ret = unsafe {
        ShellExecuteW(
            0,
            op.as_ptr(),
            file.as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            SW_SHOW,
        )
    };

    if ret <= 32 {
        bail!("ShellExecuteW failed with code {}", ret);
    }
    Ok(())
}

// ── 工具函数 ─────────────────────────────────────────────────

fn to_wide(s: &str) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    std::ffi::OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

fn url_encode(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                result.push(byte as char);
            }
            _ => {
                result.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    result
}

fn url_decode(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut bytes = s.bytes();
    while let Some(b) = bytes.next() {
        match b {
            b'%' => {
                let hi = bytes.next().unwrap_or(b'0');
                let lo = bytes.next().unwrap_or(b'0');
                if let Ok(decoded) = u8::from_str_radix(
                    &format!("{}{}", hi as char, lo as char),
                    16,
                ) {
                    result.push(decoded as char);
                }
            }
            b'+' => result.push(' '),
            _ => result.push(b as char),
        }
    }
    result
}
