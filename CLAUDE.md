# DesignGPT

将 Claude Code 打包为免安装绿色版的桌面启动器。内置便携 Node.js（node / claude / npm），通过 OAuth 2.0 PKCE 登录后一键启动，双击即可运行。

> **约定**：`$pwd` = 项目根目录（即本文件所在目录），文中所有相对路径均基于 `$pwd`。

---

## 运行环境

**你的 shell 是 PowerShell，不是 Bash。** 所有命令必须使用 PowerShell 工具执行，Bash 工具禁用。

| 场景 | 正确做法 | 错误做法 |
|------|---------|---------|
| 执行命令 | 用 **PowerShell** 工具 | ~~Bash 工具~~ |
| 路径分隔符 | 正斜杠 `/` | 反斜杠 `\` |
| 环境变量 | `$env:VAR` | `$VAR` / `%VAR%` |
| 串联命令 | `cmd1; cmd2` | `cmd1 && cmd2` |
| 列出文件 | Glob 工具 / `Get-ChildItem` | `ls` / `find` |
| 搜索内容 | Grep 工具 / `Select-String` | `grep` / `rg` |
| 读取文件 | Read 工具 | `cat` / `type` |
| 复制文件 | `Copy-Item` | `cp` / `copy` |

### 命令出错强制记录

任何命令返回非零退出码，**必须立即**追加一条记录到下方纠错表，不允许跳过。

| # | 日期 | 错误写法 | 报错关键词 | 正确写法 |
|---|------|---------|-----------|---------|
| 1 | 2026-05-13 | Bash 工具执行 `Get-ChildItem` / `Select-Object` | command not found | PowerShell 工具执行 pwsh cmdlet |
| 2 | 2026-05-13 | Bash 工具执行 `copy` | command not found | Bash 用 `cp`，或切到 PowerShell 工具 |

---

## 编译

通过 hook 自动触发：`.claude/hooks/build-hook.ps1`（`cargo build --release` → 复制 `Launcher.exe` 到根目录）

## 架构文档

| 文档 | 内容 |
|------|------|
| [docs/gui-arch.md](docs/gui-arch.md) | GUI 启动器架构：处理流程、状态机、布局、配色、关键设计决策 |
| [docs/auth.md](docs/auth.md) | OAuth 2.0 PKCE 认证：协议参数、PKCE 验证、Token 结构、端口约定 |

## Auth Server

授权服务独立部署于 `designgptserver/`（Next.js），详见其项目内 `docs/auth.md`。

| 端点 | 方法 | 职责 |
|------|------|------|
| `/authorize` | GET | 授权登录页 |
| `/authorize` | POST | 验证凭据，生成授权码，302 回 `redirect_uri` |
| `/api/auth/token` | POST | 消费授权码，验证 PKCE，签发 JWT |

## 关键设计决策

| 决策 | 原因 |
|------|------|
| OAuth 回执用 `TcpListener` 而非 Node.js | 零依赖、零进程管理、EXE 不膨胀 |
| 回调端口固定 8765 | redirect_uri 可预注册，无 TOCTOU 竞态 |
| v2 移除 Git Bash 依赖 | 用 PowerShell 原生启动 claude.exe，减少 ~50MB 体积，消除 bash 中间层 |
| Job Object 而非手动 kill | 崩溃时 OS 保证清理，不留孤儿进程 |
| 环境自检不 spawn 进程 | 避免控制台窗口闪烁 |
| 登录在独立线程 | 不阻塞 GUI |
| 启动失败 MessageBoxW 弹窗 | GUI 隐藏后用户仍能看到错误 |

## 端口

| 端口 | 用途 |
|------|------|
| `3000` | Auth Server |
| `8765` | Launcher OAuth 回调 |
