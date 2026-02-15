use std::path::PathBuf;

/// Returns the path where session specific data should be stored.
///
/// Session path is located at these locations:
///
/// | Platform | Location               |
/// | -------- | ---------------------- |
/// | Linux    | `/tmp/opie/`           |
/// | macOS    | `/tmp/opie/`           |
/// | Windows  | `%localappdata%\opie\` |
pub fn session_path() -> PathBuf {
    #[cfg(target_os = "windows")]
    return dirs::data_local_dir()
        .expect("Failed to get %localappdata%")
        .join("/opie/");

    #[cfg(not(target_os = "windows"))]
    return "/tmp/opie/".into();
}
