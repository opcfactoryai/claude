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
