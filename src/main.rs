#![windows_subsystem = "windows"]

// DesignGPT GUI Launcher — 处理流程（修改此流程必须同步更新此图）
//
// Launcher.exe
//   │
//   ├─ 1. 定位根目录（exe 所在目录）
//   ├─ 2. 创建 Job Object (JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE)
//   │      └─ launcher 退出时 OS 自动终止所有子进程，杜绝孤儿进程
//   ├─ 3. 加载中文字体 → 显示 GUI (egui, 480×620, 不可缩放, 首帧后可见)
//   └─ 4. 进入 GUI 主循环
//        │
//        ├─ [账户卡片]
//        │    ├─ 未登录 ─ "请点击下方登录按钮..."
//        │    ├─ 登录中 ─ "请在浏览器中完成授权，完成后即可启动" (绿色)
//        │    └─ 已登录 ─ 用户名 · 手机号 · 有效期
//        │
//        ├─ [登  录] ──→ oauth::login() (独立线程, 不阻塞 UI)
//        │    │
//        │    ├─ 生成 PKCE (verifier + challenge + state)
//        │    ├─ TcpListener::bind("127.0.0.1:8765")    ← 固定端口, std::net 零依赖
//        │    ├─ ShellExecuteW 打开浏览器 → 用户授权
//        │    ├─ TcpListener::accept() 阻塞等待    GET /cb?code=&state=
//        │    │      返回 HTML "登录成功"
//        │    ├─ 验证 state → ureq POST code 换 token
//        │    ├─ 解析 id_token → UserInfo
//        │    └─ 写入 .claude/settings.json → Arc<Mutex> 通知 GUI
//        │
//        └─ [启  动] ──→ 仅 [已登录] 可点击
//             │
//             ├─ GUI 隐藏 ViewportCommand::Visible(false)
//             └─ launch_claude_via_bat() (独立线程)
//                  ├─ cmd.exe /c Launcher.bat
//                  ├─ bat 负责 env 自检 + PATH 注入 + 启动 claude
//                  └─ child.wait() → std::process::exit(code)
//                       └─ Job Object 终止所有残留子进程

mod app;
mod auth;
mod auth_check;
mod job;
mod launcher;
mod oauth;

use app::DesignGPTApp;
use eframe::egui;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 绑定 Job Object：launcher 退出时 OS 自动终止所有子进程，杜绝孤儿进程
    let _job = job::JobObject::create();

    let root = env::current_exe()?
        .parent()
        .ok_or("cannot determine exe directory")?
        .to_path_buf();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([480.0, 620.0])
            .with_resizable(false)
            .with_visible(false), // 等首帧渲染完毕再显示，消除黑闪
        centered: true,
        ..Default::default()
    };

    eframe::run_native(
        "DesignGPT Launcher",
        native_options,
        Box::new(move |cc| {
            setup_chinese_font(&cc.egui_ctx);
            Ok(Box::new(DesignGPTApp::new(root)))
        }),
    )?;

    Ok(())
}

// ── 加载 Windows 系统中文字体 ─────────────────────────────────

fn setup_chinese_font(ctx: &egui::Context) {
    let font_data = match load_chinese_font() {
        Some(data) => data,
        None => return, // 没有中文字体，用默认字体（中文会显示方块）
    };

    let mut fonts = egui::FontDefinitions::default();

    // 将中文字体插入字体列表，优先级高于默认字体
    fonts
        .font_data
        .insert("chinese".to_owned(), std::sync::Arc::new(font_data));

    // Proportion 字体族：中文字体优先
    if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
        family.insert(0, "chinese".to_owned());
    }

    // Monospace 也加上
    if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
        family.insert(0, "chinese".to_owned());
    }

    ctx.set_fonts(fonts);
}

fn load_chinese_font() -> Option<egui::FontData> {
    let font_paths = [
        r"C:\Windows\Fonts\msyh.ttc",   // Microsoft YaHei (TTC)
        r"C:\Windows\Fonts\simhei.ttf",  // SimHei 黑体 (TTF)
        r"C:\Windows\Fonts\msyhbd.ttf",  // Microsoft YaHei Bold (TTF)
        r"C:\Windows\Fonts\simsun.ttc",  // SimSun 宋体 (TTC)
    ];

    for path in &font_paths {
        if let Ok(data) = std::fs::read(path) {
            return Some(egui::FontData::from_owned(data));
        }
    }

    None
}
