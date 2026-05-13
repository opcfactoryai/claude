# DesignGPT GUI Launcher 架构

## 概述

`Launcher.exe` 是一个免安装绿色版桌面启动器，内置便携 Node.js（node / claude / npm）。用户必须先完成 OAuth 2.0 PKCE 登录，才能点击「启动」按钮拉起 Claude Code。

**核心设计原则：Launcher 启动时对 `.claude/` 目录只读，仅在校验通过后进入登录状态。**

## 处理流程

```
Launcher.exe
  │
  ├─ 1. 定位根目录（exe 所在目录）
  ├─ 2. 创建 Job Object (JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE)
  │      └─ launcher 退出/崩溃时 OS 自动终止所有子进程，杜绝孤儿进程
  ├─ 3. 加载 Windows 系统中文字体（msyh / simhei / simsun）
  ├─ 4. 显示 GUI (egui, 480×620, 不可缩放, 首帧渲染完毕后 visible=true)
  ├─ 5. 环境自检（不 spawn 进程，仅检查文件存在性）
  │      ├─ node/node.exe 存在？
  │      ├─ Git/bin/bash.exe 存在？
  │      └─ 构建注入 PATH = Git\bin;node;<系统PATH>
  ├─ 6. verify_execution() —— spawn node -v / bash --version 确认二进制可执行
  │      └─ CREATE_NO_WINDOW 标志避免控制台闪烁
  ├─ 7. auth_check::check_login() 校验 .claude/auth.json 结构完整性
  │      ├─ 六项必须齐全：access_token / refresh_token / id_token /
  │      │   token_expiry / gateway
  │      ├─ id_token 的 JWT exp 未过期
  │      ├─ 全部通过 → LoggedIn（跳过登录，可直接启动）
  │      └─ 任一不通过 → LoggedOut（显示登录按钮）
  └─ 8. 进入 GUI 主循环
       │
       ├─ [账户卡片]
       │    ├─ 未登录 ─ "请点击下方登录按钮跳转至官网完成授权"
       │    ├─ 登录中 ─ "请在浏览器中完成授权，完成后即可启动" (绿色)
       │    └─ 已登录 ─ 手机号 · 余额
       │
       ├─ [登  录] ──→ oauth::login() (独立线程, 不阻塞 UI)
       │    │
       │    ├─ 生成 PKCE (verifier + challenge + state)
       │    ├─ TcpListener::bind("127.0.0.1:8765")    ← 固定端口, std::net
       │    ├─ ShellExecuteW 打开浏览器 → Auth Server (127.0.0.1:3000)
       │    ├─ TcpListener::accept() 阻塞等待    GET /cb?code=&state=
       │    │      返回 HTML "登录成功"
       │    ├─ 验证 state → ureq POST 换 token
       │    ├─ 解析 id_token → UserInfo
       │    ├─ AuthStore::merge_tokens() 写入 .claude/auth.json
       │    │      └─ 仅写 4 个 token 字段，gateway 及其他字段原样保留
       │    └─ Arc<Mutex> 通知 GUI → LoggedIn
       │
       └─ [启  动] ──→ 仅 [已登录] 可点击
            │
            ├─ GUI 隐藏 ViewportCommand::Visible(false)
            └─ launch_claude_via_bat() (独立线程)
                 ├─ cmd.exe /c Launcher.bat
                 │      ├─ PATH = node;Git\bin;%PATH%
                 │      ├─ node -v / bash --version / claude --version 自检
                 │      ├─ CLAUDE_CONFIG_DIR = root\.claude
                 │      ├─ CLAUDE_CODE_GIT_BASH_PATH = root\Git\bin\bash.exe
                 │      └─ bash -lc "exec claude.exe"
                 └─ child.wait() → std::process::exit(code)
                      └─ Job Object 终止所有残留子进程
```

## 状态机

```
┌─────────────┐  点击登录   ┌─────────────┐  浏览器回调成功  ┌─────────────┐
│  LoggedOut  │ ──────────→ │  LoggingIn  │ ──────────────→ │  LoggedIn   │
│  (初始状态)  │ ←────────── │  (等待回调)  │                 │  (可启动)    │
└─────────────┘   失败/取消   └─────────────┘                 └──────┬──────┘
      ↑                                                             │ 点击启动
      │ 启动时 auth_check 失败                                       ▼
      │ (结构不完整/过期/缺gateway)                         ┌─────────────┐
      │                                                     │  Launching  │
      └─── 启动时 auth_check 通过 ─────────────────────────→ │  (隐藏窗口  │
          (跳过登录，直接 LoggedIn)                           │   启动claude)│
                                                            └─────────────┘
```

实现：`LoginState` 枚举位于 `src/auth.rs`。`LoggingIn` 持有 `Arc<Mutex<Option<Result>>>`，OAuth 线程写入结果，GUI 主循环每 100ms 轮询一次。

## 启动校验流程 (auth_check.rs)

```
启动 Launcher.exe
      │
      ▼
auth_check::check_login()
      │
      ├─ 1. 读取 .claude/auth.json
      │      └─ 文件不存在 → None (LoggedOut)
      │
      ├─ 2. 结构完整性校验（六项必须全部非空/存在）
      │      ├─ access_token  → 缺失/空 → None
      │      ├─ refresh_token → 缺失/空 → None
      │      ├─ id_token      → 缺失/空 → None
      │      ├─ token_expiry  → 缺失 → None
      │      └─ gateway       → 缺失 → None
      │
      ├─ 3. id_token JWT exp 过期校验
      │      └─ exp < now → None
      │
      └─ 4. 全部通过 → Some(UserInfo)
             └─ GUI 进入 LoggedIn 状态
```

### auth.json 规范结构

```json
{
  "access_token": "eyJ...",
  "refresh_token": "eyJ...",
  "id_token": "eyJ...",
  "token_expiry": 1778502825,
  "gateway": {
    "ENABLE_GARDEN_IMAGEGEN": "1",
    "OPENAI_API_KEY": "sk-...",
    "OPENAI_BASE_URL": "https://...",
    "OPENAI_IMAGE_MODEL": "gpt-image-2"
  }
}
```

**gateway 字段保护机制**：`AuthStore::merge_tokens()` 只写入 access_token / refresh_token / id_token / token_expiry 四个 token 字段，gateway 及用户添加的自定义字段完全保留，绝不清除。

## 系统架构

```
┌─────────────────────┐       ┌──────────────────────┐
│   Launcher.exe       │       │  Auth Server (3000)   │
│                      │       │                      │
│  ┌────────────────┐  │       │  /authorize          │
│  │  GUI (egui)    │  │  OAuth│  /api/auth/token     │
│  │  ┌──────────┐  │  │◄─────►│                      │
│  │  │ 账户卡片  │  │  │       └──────────────────────┘
│  │  │ 按钮区    │  │  │
│  │  └──────────┘  │  │       ┌──────────────────────┐
│  └────────────────┘  │       │  cmd.exe              │
│                      │ spawn │  └── Launcher.bat     │
│  ┌────────────────┐  │──────►│       └── bash -lc    │
│  │  Job Object    │  │       │            └── claude │
│  └────────────────┘  │       └──────────────────────┘
│                      │
│  node/                │
│  ├── node.exe         │
│  ├── claude.exe       │
│  Git/bin/             │
│  ├── bash.exe         │
│  ├── git.exe          │
│  Launcher.bat         │
└─────────────────────┘
```

## 文件结构

```
src/
├── main.rs          # 入口：Job Object → 字体 → eframe::run_native()
├── app.rs           # egui App：状态机 + UI 渲染（暗色主题）
├── auth.rs          # AuthStore (load/merge_tokens/is_token_valid) + UserInfo + LoginState
├── auth_check.rs    # 启动时 auth.json 结构完整性 + JWT 过期校验
├── launcher.rs      # MessageBoxW 错误弹窗 + 通过 cmd.exe /c Launcher.bat 启动 Claude
├── oauth.rs         # OAuth 2.0 PKCE 登录（TcpListener 回执 + ureq HTTP）
└── job.rs           # Windows Job Object 防孤儿进程

Launcher.bat         # 启动脚本（自检环境 + PATH 注入 + 启动 claude）
```

## GUI 布局 (480×620)

```
┌──────────────────────────────────────┐
│                                      │
│           DesignGPT                   │  ← 品牌 (28px, #4fc3f7)
│                                      │
│  ┌────────────────────────────────┐  │
│  │  账户信息                       │  │  ← 卡片
│  │  ──────────────────────────── │  │
│  │  手机号：138****8000           │  │
│  │  余额：¥ 50.00                 │  │
│  └────────────────────────────────┘  │
│                                      │
│        ┌──────────────┐             │
│        │    登  录     │             │  ← 登录按钮
│        └──────────────┘             │
│        ┌──────────────┐             │
│        │    启  动     │             │  ← 启动按钮
│        └──────────────┘             │
│                                      │
│  武汉市向量求索信息技术有限公司          │  ← 底部版权 (11px)
└──────────────────────────────────────┘
```

## 配色方案 (暗色主题)

| 用途 | 色值 | 常量 |
|------|------|------|
| 主色调 | `#4fc3f7` | `ACCENT` |
| 成功 | `#66bb6a` | `ACCENT_GREEN` |
| 错误 | `#ef5350` | `ACCENT_RED` |
| 主文字 | `#e0e0e0` | `TEXT_PRIMARY` |
| 次文字 | `#9e9e9e` | `TEXT_SECONDARY` |
| 提示文字 | `#ffffff` × 0.31 | `TEXT_HINT` |
| 禁用背景 | `#37373c` | `DISABLED_BG` |
| 禁用文字 | `#78787d` | `DISABLED_TEXT` |

## 关键设计决策

| 决策 | 原因 |
|------|------|
| 启动时对 `.claude/` 只读 | 防止误写覆盖用户配置 |
| `auth_check` 六项结构校验 | access_token / refresh_token / id_token / token_expiry / gateway 全部齐全且 JWT 未过期才放行 |
| `merge_tokens` 只写 4 个 token 字段 | gateway 及用户自定义字段绝不清除（有单元测试保证） |
| 启动 Claude 通过 `Launcher.bat` | bat 负责 PATH / 环境变量 / bash -lc 完整配置，Rust 侧零依赖 |
| OAuth 回执用 `TcpListener` 而非 HTTP server | 零依赖、零进程管理、EXE 不膨胀 |
| 回调端口固定 8765 | `redirect_uri` 可预注册，消除 TOCTOU 竞态 |
| Job Object (`JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`) | 崩溃/强制关闭时 OS 保证清理，不留孤儿进程 |
| 环境自检不 spawn 进程（仅文件存在性检查） | 快速，无窗口闪烁 |
| `verify_execution` 加 `CREATE_NO_WINDOW` | 确保二进制可用，同时避免控制台窗口闪烁 |
| 登录在独立线程 | 不阻塞 GUI 主循环 |
| 启动失败 `MessageBoxW` 弹窗 | GUI 已隐藏后用户仍能看到错误原因 |
| 首帧渲染完毕后 `visible=true` | 消除窗口创建时的黑闪 |
| `windows_subsystem = "windows"` | 无控制台窗口，纯 GUI 应用 |
| release 编译 `opt-level = "z"` / `lto = true` / `strip = true` | 最小化 exe 体积 |

## 端口约定

| 端口 | 实体 | 用途 |
|------|------|------|
| `3000` | Auth Server (Next.js) | 授权页面 + Token 签发 |
| `8765` | Launcher TcpListener | 接收 OAuth 回调 |

## 依赖

```toml
[dependencies]
anyhow = "1"
eframe = "0.30"              # egui 框架（winit + wgpu 后端）
serde = { version = "1", features = ["derive"] }
serde_json = "1"
rand = "0.8"                 # PKCE state / verifier 随机生成
base64 = "0.22"              # PKCE challenge + JWT 编解码
sha2 = "0.10"                # SHA256 for PKCE
ureq = { version = "2", features = ["json"] }  # Token 交换 HTTP 客户端
```

不引入 HTTP 服务端库，OAuth 回调使用 `std::net::TcpListener` 实现极简 HTTP 回执。

## 编译

编译通过 hook 自动触发：`.claude/hooks/build-hook.ps1`

```
cargo build --release        # → target/release/Launcher.exe
hook: Copy-Item Launcher.exe → 项目根目录
```

release profile 配置 `opt-level = "z"`（体积优先）、`lto = true`（链接时优化）、`panic = "abort"`、`strip = true`（移除符号）。
