#[cfg(target_os = "windows")]
pub fn win_get_long_path_name(path: &str) -> anyhow::Result<String> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use winapi::um::fileapi::GetLongPathNameW;

    let mut buf: Vec<u16> = vec![0; 1024];
    let path_wide: Vec<u16> = OsStr::new(path).encode_wide().chain(Some(0)).collect();
    let res = unsafe { GetLongPathNameW(path_wide.as_ptr(), buf.as_mut_ptr(), buf.len() as u32) };
    if res == 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    Ok(String::from_utf16_lossy(&buf[..res as usize]))
}

#[cfg(not(target_os = "windows"))]
pub fn win_get_long_path_name(_path: &str) -> anyhow::Result<String> {
    unimplemented!();
}

#[cfg(target_os = "linux")]
fn dlerror() -> String {
    use std::ffi::CStr;

    let error = unsafe { CStr::from_ptr(libc::dlerror()) };
    error.to_string_lossy().to_string()
}

#[cfg(target_os = "linux")]
pub fn linux_find_native_glfw() -> anyhow::Result<String> {
    // reference: https://github.com/unmojang/FjordLauncher/blob/6d0109357551bc29079da18543b7db61223c7f38/launcher/MangoHud.cpp#L141
    use std::ffi::{CStr, CString};

    let name = "libglfw.so";
    let name_cstr = CString::new(name)?;
    let lib = unsafe { libc::dlopen(name_cstr.as_ptr(), libc::RTLD_NOW) };
    if lib.is_null() {
        return Err(anyhow::anyhow!(
            "Failed to find a native GLFW library: {}",
            dlerror()
        ));
    }
    let path = [0u8; libc::PATH_MAX as usize + 1];
    if unsafe { libc::dlinfo(lib, libc::RTLD_DI_ORIGIN, path.as_ptr() as _) } == 0 {
        let path = CStr::from_bytes_until_nul(&path)?
            .to_string_lossy()
            .to_string();
        Ok(path + "/" + name)
    } else {
        Err(anyhow::anyhow!(
            "Failed to get the path of the native GLFW library: {}",
            dlerror()
        ))
    }
}
