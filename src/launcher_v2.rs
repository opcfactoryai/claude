use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::Command;

/// 通过 Launcher-v2.bat 启动 Claude。
///
/// bat / ps1 负责所有环境设置：
///   1. PATH = node;%PATH%
///   2. 检查 node -v / claude --version
///   3. 设置 CLAUDE_CONFIG_DIR
///   4. 直接执行 claude.exe（Git-free，无 bash 依赖）
///
/// 此函数阻塞直到 bat / Claude 退出，返回 exit code。
pub fn launch_claude_via_bat(root: &PathBuf) -> Result<i32> {
    let bat = root.join("Launcher-v2.bat");
    if !bat.exists() {
        anyhow::bail!(
            "Launcher-v2.bat 未找到: {}\n请确保 Launcher-v2.bat 与 Launcher.exe 在同一目录",
            bat.display()
        );
    }

    let status = Command::new("cmd.exe")
        .args(["/c", "Launcher-v2.bat"])
        .current_dir(root)
        .spawn()
        .context("无法启动 Launcher-v2.bat，请确认 cmd.exe 可用")?
        .wait()
        .context("等待 Launcher-v2.bat 退出时出错")?;

    Ok(status.code().unwrap_or(1))
}
