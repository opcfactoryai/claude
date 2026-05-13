//  Windows Job Object — 进程生命周期管理
//
// 将当前进程及其所有子进程绑定到一个 Job Object，
// 设置 JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE。
// 当 launcher 退出时（无论正常还是崩溃），OS 自动关闭 job handle，
// 触发强制终止所有子进程，杜绝孤儿进程。

#[allow(non_camel_case_types, non_snake_case)]
mod ffi {
    pub type BOOL = i32;
    pub type DWORD = u32;
    pub type HANDLE = isize;
    pub type LPVOID = *mut std::ffi::c_void;
    pub type SIZE_T = usize;

    pub const JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE: DWORD = 0x00002000;

    #[repr(C)]
    pub struct JOBOBJECT_BASIC_LIMIT_INFORMATION {
        pub PerProcessUserTimeLimit: i64,
        pub PerJobUserTimeLimit: i64,
        pub LimitFlags: DWORD,
        pub MinimumWorkingSetSize: SIZE_T,
        pub MaximumWorkingSetSize: SIZE_T,
        pub ActiveProcessLimit: DWORD,
        pub Affinity: SIZE_T,
        pub PriorityClass: DWORD,
        pub SchedulingClass: DWORD,
    }

    #[repr(C)]
    pub struct JOBOBJECT_EXTENDED_LIMIT_INFORMATION {
        pub BasicLimitInformation: JOBOBJECT_BASIC_LIMIT_INFORMATION,
        pub IoInfo: [u8; 48],
        pub ProcessMemoryLimit: SIZE_T,
        pub JobMemoryLimit: SIZE_T,
        pub PeakProcessMemoryUsed: SIZE_T,
        pub PeakJobMemoryUsed: SIZE_T,
    }

    pub const JOB_OBJECT_INFO_EXTENDED_LIMIT: i32 = 9;

    #[link(name = "kernel32")]
    extern "system" {
        pub fn CreateJobObjectW(
            lpJobAttributes: *mut std::ffi::c_void,
            lpName: *const u16,
        ) -> HANDLE;

        pub fn SetInformationJobObject(
            hJob: HANDLE,
            JobObjectInfoClass: i32,
            lpJobObjectInfo: LPVOID,
            cbJobObjectInfoLength: DWORD,
        ) -> BOOL;

        pub fn AssignProcessToJobObject(hJob: HANDLE, hProcess: HANDLE) -> BOOL;

        pub fn GetCurrentProcess() -> HANDLE;

        pub fn CloseHandle(hObject: HANDLE) -> BOOL;
    }
}

use ffi::*;

pub struct JobObject {
    handle: HANDLE,
}

impl JobObject {
    /// 创建 Job Object 并将当前进程绑定。
    /// 返回 None 表示创建失败（不影响程序继续运行，只是失去孤儿进程保护）。
    pub fn create() -> Option<Self> {
        unsafe {
            let handle = CreateJobObjectW(std::ptr::null_mut(), std::ptr::null());
            if handle == 0 {
                return None;
            }

            let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
            info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;

            let size = std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as DWORD;
            let ok = SetInformationJobObject(
                handle,
                JOB_OBJECT_INFO_EXTENDED_LIMIT,
                &mut info as *mut _ as LPVOID,
                size,
            );
            if ok == 0 {
                CloseHandle(handle);
                return None;
            }

            let ok = AssignProcessToJobObject(handle, GetCurrentProcess());
            if ok == 0 {
                CloseHandle(handle);
                return None;
            }

            Some(Self { handle })
        }
    }
}

// 进程正常退出时 Drop 显式关闭 handle；
// 崩溃时 OS 自动关闭所有 handle，触发 KILL_ON_JOB_CLOSE 终止子进程。
impl Drop for JobObject {
    fn drop(&mut self) {
        if self.handle != 0 {
            unsafe {
                CloseHandle(self.handle);
            }
            self.handle = 0;
        }
    }
}
