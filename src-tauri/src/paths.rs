use std::path::{Path, PathBuf};
// Manager is a tauri trait that unlocks the .path() method on AppHandle
// and apparently without this import, the compiler cant see it even though AppHandle is in scope. go figure
use tauri::Manager;

// tauri bundles the binaries into resource_dir at build time (see tauri.conf.json bundle.resources)
// these two functions are the single place where the rest of the app gets a path to them

// AppHandle is tauris runtime handle to the app, i need it to ask tauri where it put the files
// Result<PathBuf, String> means it either returns a valid path or an error message
pub fn yt_dlp_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    // windows needs the .exe suffix, otherwise the spawn will silently fail
    #[cfg(target_os = "windows")]
    let name = "yt-dlp.exe";
    #[cfg(not(target_os = "windows"))]
    let name = "yt-dlp";

    resolve_binary(app, name)
}

pub fn ffmpeg_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    #[cfg(target_os = "windows")]
    let name = "ffmpeg.exe";
    #[cfg(not(target_os = "windows"))]
    let name = "ffmpeg";

    resolve_binary(app, name)
}

// private, only yt_dlp_path and ffmpeg_path should go through here
fn resolve_binary(app: &tauri::AppHandle, name: &str) -> Result<PathBuf, String> {
    let path = app
        .path()
        .resource_dir() // resolves to the folder where tauri copies the bundled files at install time
        .map_err(|e| e.to_string())? // converts tauris error type into a plain String, for simple return.
        // the ? at the end means: if this failed, return the error now instead of continuing. sweet syntactic sugar.
        .join("vendor")
        .join(name);

    if !path.exists() {
        return Err(format!("binary not found: {}", path.display()));
    }

    // bundled files can lose their execute bit during packaging, fix it before trying to spawn
    // unix only, windows just ignores the execute bit and relies on file extensions instead. rare windows w? maybe.
    #[cfg(unix)]
    ensure_executable(&path)?;

    Ok(path)
}

// it follows that there is no point compiling this on windows
#[cfg(unix)]
fn ensure_executable(path: &Path) -> Result<(), String> {
    // these two imports live here rather than at the top because they are unix only. bad practice?
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    // metadata gives the files inode info, i only need the permissions part of it
    let mut perms = fs::metadata(path)
        .map_err(|e| e.to_string())?
        .permissions();

    // unix permissions are a bitmask, 0o111 is the execute bit for owner, group and others
    // if none of those three bits are set the file isnt executable by anyone
    // only write if the bit is actually missing, avoids unnecessary fs writes
    if perms.mode() & 0o111 == 0 {
        // OR in the execute bits without touching read or write
        perms.set_mode(perms.mode() | 0o111);
        fs::set_permissions(path, perms).map_err(|e| e.to_string())?;
    }

    Ok(())
}