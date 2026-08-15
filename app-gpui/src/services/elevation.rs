//! 检测 / 以管理员重启（Windows 脚本模拟输入常需与游戏同级权限）。

use std::sync::OnceLock;

/// 启动时若需提升但失败，供 UI 展示原因（`ensure_elevated_if_configured` 写入）。
pub static STARTUP_ELEVATION_ERROR: OnceLock<String> = OnceLock::new();

#[cfg(windows)]
pub fn is_elevated() -> bool {
    is_elevated::is_elevated()
}

#[cfg(not(windows))]
pub fn is_elevated() -> bool {
    false
}

#[cfg(windows)]
fn os_str_wide(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    s.encode_wide().chain(std::iter::once(0)).collect()
}

#[cfg(windows)]
fn path_wide(path: &std::path::Path) -> Vec<u16> {
    os_str_wide(path.as_os_str())
}

#[cfg(windows)]
fn shell_execute_runas(exe: &std::path::Path, work_dir: &std::path::Path) -> Result<(), String> {
    use std::ffi::{c_void, OsStr};
    use std::ptr;

    let exe_wide = path_wide(exe);
    let verb_wide = os_str_wide(OsStr::new("runas"));
    let dir_wide = path_wide(work_dir);

    #[repr(C)]
    struct ShellExecuteInfoW {
        cb_size: u32,
        f_mask: u32,
        hwnd: *mut c_void,
        lp_verb: *const u16,
        lp_file: *const u16,
        lp_parameters: *const u16,
        lp_directory: *const u16,
        n_show: i32,
        h_inst_app: *mut c_void,
        lp_id_list: *mut c_void,
        lp_class: *const u16,
        hkey_class: *mut c_void,
        dw_hot_key: u32,
        h_monitor: *mut c_void,
        h_process: *mut c_void,
    }

    const SEE_MASK_NOCLOSEPROCESS: u32 = 0x00000040;
    const SW_SHOW: i32 = 5;
    const SE_ERR_CANCELLED: usize = 1223;

    #[link(name = "shell32")]
    unsafe extern "system" {
        fn ShellExecuteExW(p_exec_info: *mut ShellExecuteInfoW) -> i32;
    }

    let mut info = ShellExecuteInfoW {
        cb_size: std::mem::size_of::<ShellExecuteInfoW>() as u32,
        f_mask: SEE_MASK_NOCLOSEPROCESS,
        hwnd: ptr::null_mut(),
        lp_verb: verb_wide.as_ptr(),
        lp_file: exe_wide.as_ptr(),
        lp_parameters: ptr::null(),
        lp_directory: dir_wide.as_ptr(),
        n_show: SW_SHOW,
        h_inst_app: ptr::null_mut(),
        lp_id_list: ptr::null_mut(),
        lp_class: ptr::null(),
        hkey_class: ptr::null_mut(),
        dw_hot_key: 0,
        h_monitor: ptr::null_mut(),
        h_process: ptr::null_mut(),
    };

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetLastError() -> u32;
        fn CloseHandle(h: *mut c_void) -> i32;
    }

    let ok = unsafe { ShellExecuteExW(&mut info) };
    if ok == 0 {
        let code = unsafe { GetLastError() };
        return Err(format!("请求管理员权限失败 (错误码 {code})"));
    }

    let hinst = info.h_inst_app as usize;
    if hinst == 0 || hinst <= 32 || hinst == SE_ERR_CANCELLED {
        return Err(match hinst {
            0 => "系统资源不足，无法请求管理员权限".into(),
            2 => "未找到程序文件".into(),
            3 => "路径无效".into(),
            5 => "拒绝访问，无法以管理员启动".into(),
            SE_ERR_CANCELLED => "已取消管理员权限请求（UAC）".into(),
            _ => format!("请求管理员权限失败 (错误码 {hinst})"),
        });
    }

    if !info.h_process.is_null() {
        unsafe {
            CloseHandle(info.h_process);
        }
    }

    Ok(())
}

#[cfg(windows)]
pub fn restart_as_admin() -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| format!("获取程序路径失败: {e}"))?;
    let work_dir = exe
        .parent()
        .map(std::path::Path::to_path_buf)
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    shell_execute_runas(&exe, &work_dir)
}

#[cfg(not(windows))]
pub fn restart_as_admin() -> Result<(), String> {
    Err("仅 Windows 支持以管理员身份重启".into())
}

/// 启动早期：若配置要求管理员且当前未提升，则弹出 UAC 并拉起提升实例后退出。
pub fn ensure_elevated_if_configured(launch_as_administrator: bool) {
    if !launch_as_administrator {
        return;
    }
    if is_elevated() {
        return;
    }
    match restart_as_admin() {
        Ok(()) => std::process::exit(0),
        Err(e) => {
            let _ = STARTUP_ELEVATION_ERROR.set(e);
        }
    }
}

/// 启动后 Toast：仅在提升失败或未获得管理员权限时提示。
pub fn startup_elevation_toast(launch_as_administrator: bool) -> Option<String> {
    if let Some(err) = STARTUP_ELEVATION_ERROR.get() {
        return Some(format!(
            "已开启「以管理员启动」，但提升失败：{err}。请在设置中点击「立即以管理员重启」。"
        ));
    }
    if launch_as_administrator && !is_elevated() {
        Some(
            "已开启「以管理员启动」，但当前未获得管理员权限，自动演奏可能无效。请在设置中点击「立即以管理员重启」或在 UAC 中允许提升。"
                .into(),
        )
    } else {
        None
    }
}
