use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::Command;

// ── 错误弹窗 (MessageBoxW) ────────────────────────────────────

#[link(name = "user32")]
extern "system" {
    fn MessageBoxW(
        hwnd: isize,
        lpText: *const u16,
        lpCaption: *const u16,
        uType: u32,
    ) -> i32;
}

const MB_ICONERROR: u32 = 0x00000010;

fn to_wide(s: &str) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    std::ffi::OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

pub fn show_launch_error(reason: &str) {
    let text = to_wide(&format!("启动失败：{}\n\n请检查运行环境是否完整。", reason));
    let caption = to_wide("DesignGPT 启动错误");
    unsafe {
        MessageBoxW(0, text.as_ptr(), caption.as_ptr(), MB_ICONERROR);
    }
}

// ── 启动 Claude ──────────────────────────────────────────────

/// 通过 Launcher.bat 启动 Claude。
///
/// bat / ps1 负责所有环境设置：
///   1. PATH = node;%PATH%
///   2. 检查 node -v / claude --version
///   3. 设置 CLAUDE_CONFIG_DIR
///   4. 直接执行 claude.exe
///
/// 此函数阻塞直到 bat / Claude 退出，返回 exit code。
pub fn launch_claude_via_bat(root: &PathBuf) -> Result<i32> {
    let bat = root.join("Launcher.bat");
    if !bat.exists() {
        anyhow::bail!(
            "Launcher.bat 未找到: {}\n请确保 Launcher.bat 与 Launcher.exe 在同一目录",
            bat.display()
        );
    }

    let status = Command::new("cmd.exe")
        .args(["/c", "Launcher.bat"])
        .current_dir(root)
        .spawn()
        .context("无法启动 Launcher.bat，请确认 cmd.exe 可用")?
        .wait()
        .context("等待 Launcher.bat 退出时出错")?;

    Ok(status.code().unwrap_or(1))
}
