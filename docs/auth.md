# OAuth 2.0 PKCE 认证架构

## 参与者

| 角色 | 实体 | 地址 |
|------|------|------|
| Client | Launcher.exe | `127.0.0.1:8765` (TcpListener 回调) |
| Auth Server | Next.js (designgptserver) | `127.0.0.1:3000` |
| User | 浏览器 | — |

## 协议参数

| 参数 | 生成方 | 说明 |
|------|--------|------|
| `code_verifier` | Launcher | 随机字符串 43-128 字符，字符集 `[A-Za-z0-9-._~]` |
| `code_challenge` | Launcher | `base64url(SHA256(code_verifier))`，无 padding |
| `state` | Launcher | 随机串 16-32 字符，字符集 `[0-9A-Za-z]`，防 CSRF |
| `code` | Auth Server | 一次性授权码，5min 过期 |
| `access_token` | Auth Server | JWT (HS256)，2h 过期 |
| `id_token` | Auth Server | JWT (HS256)，含用户信息，2h 过期 |
| `refresh_token` | Auth Server | JWT (HS256)，30d 过期 |

## 完整流程

```
Launcher (8765)              Browser              Auth Server (3000)
═══════════════              ═══════              ════════════════
     │                           │                       │
     │ ① 生成 PKCE 参数          │                       │
     │   code_verifier           │                       │
     │   code_challenge          │                       │
     │   state                   │                       │
     │                           │                       │
     │ ② TcpListener::bind       │                       │
     │   ("127.0.0.1:8765")      │                       │
     │                           │                       │
     │ ③ ShellExecuteW 打开浏览器 │                       │
     │ ─────────────────────────→│                       │
     │                           │ GET /authorize?       │
     │                           │   response_type=code  │
     │                           │   client_id=           │
     │                           │    designgpt-desktop   │
     │                           │   redirect_uri=        │
     │                           │    http://127.0.       │
     │                           │    0.1:8765/cb         │
     │                           │   code_challenge=...   │
     │                           │   code_challenge_      │
     │                           │    method=S256         │
     │                           │   state=...            │
     │                           │   scope=openid         │
     │                           │    %20profile          │
     │                           │ ──────────────────────→│
     │                           │                       │ ④ 授权登录页
     │                           │ ←─────────────────────│   手机号 + 验证码
     │                           │                       │
     │                           │ ⑤ POST /authorize     │
     │                           │   提交凭据              │
     │                           │ ──────────────────────→│
     │                           │                       │ ⑥ 验证凭据
     │                           │                       │   生成授权码
     │                           │                       │
     │                           │ ⑦ 302 → /cb?code=    │
     │                           │   &state=             │
     │                           │ ←─────────────────────│
     │                           │                       │
     │ ⑧ TcpListener::accept()   │                       │
     │ ←─────────────────────────│                       │
     │   解析 GET /cb?code=&state=                        │
     │   返回 200 OK + 成功 HTML                          │
     │   非 /cb 请求 → 404 后继续 accept()                 │
     │                           │                       │
     │ ⑨ 验证 state 一致性        │                       │
     │   state != expected → bail │                       │
     │                           │                       │
     │ ⑩ POST /api/auth/token    │                       │
     │ ──────────────────────────────────────────────────→│
     │   grant_type=                                      │ ⑪ 消费授权码
     │    authorization_code                              │   验证 PKCE:
     │   code=XXX                                         │   base64url(
     │   redirect_uri=...                                 │    SHA256(
     │   client_id=                                       │     code_verifier
     │    designgpt-desktop                               │   )) ==
     │   code_verifier=...                                │   code_challenge
     │                                                    │   签发 JWT
     │ ←──────────────────────────────────────────────────│
     │   {access_token, id_token,                         │
     │    refresh_token, expires_in}                      │
     │                                                    │
     │ ⑫ id_token → UserInfo                              │
     │   AuthStore::merge_tokens(root)                    │
     │   写入 .claude/auth.json                            │
     │   GUI → LoggedIn                                   │
```

## PKCE 验证

```
Server 端验证:
  base64url(SHA256(code_verifier)) == code_challenge
```

`code_verifier` 仅 Launcher 持有，Auth Server 从未收到明文。`code_challenge` 通过授权请求参数传递，SHA256 单向不可逆。

## OAuth 配置

配置通过环境变量注入，均有默认值（`oauth.rs` — `OAuthConfig::default()`）：

| 参数 | 环境变量 | 默认值 |
|------|---------|--------|
| 授权端点 | `OAUTH_AUTHORIZE_URL` | `http://127.0.0.1:3000/authorize` |
| Token 端点 | `OAUTH_TOKEN_URL` | `http://127.0.0.1:3000/api/auth/token` |
| Client ID | `OAUTH_CLIENT_ID` | `designgpt-desktop` |
| Scope | `OAUTH_SCOPES` | `openid profile` |

## 端口约定

| 端口 | 用途 |
|------|------|
| `3000` | Auth Server（授权页面 + Token 签发） |
| `8765` | Launcher `TcpListener` 回调监听（固定，`redirect_uri` 可预注册） |

## Token 结构

### access_token / id_token JWT Payload

```json
{
  "sub": "13800138000",
  "phone_number": "13800138000",
  "name": "DesignGPT用户",
  "nickname": "DesignGPT用户",
  "exp": 1715400000
}
```

### 本地校验与解析

Launcher 解析 `id_token` 的 JWT payload → `UserInfo`：
- `phone`: `phone_number` 或 `sub` 字段，脱敏显示（`138****8000`，`mask_phone()` 函数）
- `name`: `nickname` 或 `name`
- `balance`: 固定显示 `¥ 50.00`（待后续接入余额接口）

`access_token` 的 JWT `exp` 用于 `is_token_valid()` 判断登录态是否过期。

JWT 解析不验证签名（HMAC 密钥仅 Auth Server 持有），仅解码 payload 提取用户信息和过期时间。

## 存储

```
.claude/auth.json
{
  "access_token": "eyJ...",
  "id_token": "eyJ...",
  "refresh_token": "eyJ...",
  "token_expiry": 1715400000,
  "gateway": {
    "ENABLE_GARDEN_IMAGEGEN": "1",
    "OPENAI_API_KEY": "sk-...",
    "OPENAI_BASE_URL": "https://...",
    "OPENAI_IMAGE_MODEL": "gpt-image-2"
  }
}
```

**写入策略** — `AuthStore::merge_tokens()`（`auth.rs`）：
- 仅写入 `access_token` / `refresh_token` / `id_token` / `token_expiry` 四个字段
- `gateway` 及用户在 auth.json 中添加的任何自定义字段完全保留
- 首次登录（auth.json 不存在）时创建新文件，不含 gateway

**读取策略** — `AuthStore::load()`（`auth.rs`）：
- 文件不存在 → 返回 `AuthStore::default()`
- 反序列化失败 → 返回 error

**校验策略** — `auth_check::check_login()`（`auth_check.rs`）：
- 启动时唯一入口，只读不写
- 六项缺一不可：`access_token` / `refresh_token` / `id_token` / `token_expiry` / `gateway` 全部非空 + JWT 未过期
- 通过 → `Some(UserInfo)`，失败 → `None`

## 错误处理

| 阶段 | 错误类型 | 处理方式 |
|------|---------|---------|
| TcpListener::bind | 端口被占用 | `bail!` → GUI 显示红色错误卡片 |
| ShellExecuteW | 返回值 ≤ 32 | `bail!` 并带错误码 |
| TcpListener::accept | IO 错误 | `bail!` → GUI 显示错误 |
| state 校验 | state 不匹配 | `bail!` 带 expected/got 详情 |
| Token 交换 | HTTP 错误 / 超时 | `bail!` 带 ureq 错误信息 |
| Token 响应 | `error` 字段非空 | `bail!` 带 `error_description` |
| Token 响应 | 缺少 `access_token` | `bail!` "missing access_token" |

所有错误均通过 `Arc<Mutex<Option<Result>>>` 传回 GUI 主线程，显示在红色错误卡片中。

## 相关文件

| 项目 | 文件 | 职责 |
|------|------|------|
| Launcher | `src/oauth.rs` | PKCE 生成、浏览器打开、TCP 回调、Token 交换 |
| Launcher | `src/auth.rs` | AuthStore 存储/解析、UserInfo、LoginState 状态机 |
| Launcher | `src/auth_check.rs` | 启动时 auth.json 完整性校验 |
| Launcher | `src/app.rs` | GUI 登录状态驱动、OAuth 线程发起 |
| Server | `designgptserver/` | Next.js Auth Server（授权页面 + Token 签发） |
