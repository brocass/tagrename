//! Windows shell integration helpers.

/// Open a path with the user's default associated application.
///
/// Uses `ShellExecuteW` directly instead of `cmd /C start`, which can flash
/// a command prompt window before the media player opens.
#[cfg(windows)]
pub fn open_default(path: &std::path::Path) -> Result<(), String> {
    use std::ffi::OsStr;
    use std::iter::once;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr;

    type Hinstance = isize;
    type Hwnd = isize;

    const SW_SHOWNORMAL: i32 = 1;

    #[link(name = "shell32")]
    extern "system" {
        fn ShellExecuteW(
            hwnd: Hwnd,
            lp_verb: *const u16,
            lp_file: *const u16,
            lp_parameters: *const u16,
            lp_directory: *const u16,
            n_show: i32,
        ) -> Hinstance;
    }

    let file = path.as_os_str();
    let file_w: Vec<u16> = OsStr::new(file).encode_wide().chain(once(0)).collect();
    let verb_w: Vec<u16> = OsStr::new("open").encode_wide().chain(once(0)).collect();

    let result = unsafe {
        ShellExecuteW(
            0,
            verb_w.as_ptr(),
            file_w.as_ptr(),
            ptr::null(),
            ptr::null(),
            SW_SHOWNORMAL,
        )
    };

    // ShellExecuteW returns a value greater than 32 on success, otherwise
    // it encodes an error code. SEE_MASK_FLAG_NO_UI is not passed here because
    // it would suppress useful "no app associated" messages from Windows.
    if result > 32 {
        Ok(())
    } else {
        Err(format!("Windows shell error {result}"))
    }
}

#[cfg(not(windows))]
pub fn open_default(_path: &std::path::Path) -> Result<(), String> {
    Err("Windows shell integration is unavailable on this platform".into())
}
