use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use serde::Serialize;
use std::fs;
use std::path::Path;
use std::time::UNIX_EPOCH;

#[derive(Debug, Serialize)]
pub struct FileStatExtended {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: u64,
    pub created: u64,
    pub extension: String,
    pub readonly: bool,
    pub is_symlink: bool,
}

#[derive(Debug, Serialize, Clone)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: u64,
    pub extension: String,
}

#[derive(Debug, Serialize)]
pub struct DefaultDirs {
    pub home: String,
    pub documents: String,
    pub downloads: String,
    pub desktop: String,
}

fn path_to_string(p: &Path) -> String {
    p.to_string_lossy().to_string()
}

fn build_entry(path: &Path) -> Result<FileEntry, String> {
    let meta = fs::metadata(path).map_err(|e| format!("Failed to stat {}: {}", path.display(), e))?;
    let modified = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    let extension = path
        .extension()
        .map(|e| e.to_string_lossy().to_string())
        .unwrap_or_default();

    Ok(FileEntry {
        name,
        path: path_to_string(path),
        is_dir: meta.is_dir(),
        size: meta.len(),
        modified,
        extension,
    })
}

#[tauri::command]
pub fn fs_read_dir(path: String) -> Result<Vec<FileEntry>, String> {
    let entries = fs::read_dir(&path).map_err(|e| format!("Failed to read dir: {}", e))?;
    let mut result = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let entry_path = entry.path();
        // Skip hidden files/dirs (starting with .)
        if let Some(name) = entry_path.file_name() {
            if name.to_string_lossy().starts_with('.') {
                continue;
            }
        }
        match build_entry(&entry_path) {
            Ok(fe) => result.push(fe),
            Err(_) => continue, // Skip entries we can't stat
        }
    }
    Ok(result)
}

#[tauri::command]
pub fn fs_read_file_text(path: String) -> Result<String, String> {
    fs::read_to_string(&path).map_err(|e| format!("Failed to read file: {}", e))
}

#[tauri::command]
pub fn fs_write_file(path: String, content: String) -> Result<(), String> {
    fs::write(&path, content).map_err(|e| format!("Failed to write file: {}", e))
}

#[tauri::command]
pub fn fs_delete(path: String, recursive: bool) -> Result<(), String> {
    let p = Path::new(&path);
    if p.is_dir() {
        if recursive {
            fs::remove_dir_all(p).map_err(|e| format!("Failed to delete dir: {}", e))
        } else {
            fs::remove_dir(p).map_err(|e| format!("Failed to delete dir: {}", e))
        }
    } else {
        fs::remove_file(p).map_err(|e| format!("Failed to delete file: {}", e))
    }
}

#[tauri::command]
pub fn fs_rename(old_path: String, new_path: String) -> Result<(), String> {
    fs::rename(&old_path, &new_path).map_err(|e| format!("Failed to rename: {}", e))
}

#[tauri::command]
pub fn fs_stat(path: String) -> Result<FileEntry, String> {
    build_entry(Path::new(&path))
}

#[tauri::command]
pub fn fs_create_dir(path: String) -> Result<(), String> {
    fs::create_dir_all(&path).map_err(|e| format!("Failed to create dir: {}", e))
}

#[tauri::command]
pub fn fs_exists(path: String) -> Result<bool, String> {
    Ok(Path::new(&path).exists())
}

#[tauri::command]
pub fn fs_read_file_base64(path: String) -> Result<String, String> {
    let p = Path::new(&path);
    let data = fs::read(p).map_err(|e| format!("Failed to read file: {}", e))?;
    let ext = p
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    let mime = match ext.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "bmp" => "image/bmp",
        "ico" => "image/x-icon",
        "tiff" | "tif" => "image/tiff",
        "pdf" => "application/pdf",
        _ => "application/octet-stream",
    };
    let b64 = BASE64.encode(&data);
    Ok(format!("data:{};base64,{}", mime, b64))
}

/// Resolve a (possibly relative) path against an optional base directory.
/// Returns the canonicalized absolute path string.
#[tauri::command]
pub fn fs_resolve_path(path: String, base_dir: Option<String>) -> Result<String, String> {
    let p = Path::new(&path);
    let resolved = if p.is_absolute() {
        p.to_path_buf()
    } else if let Some(ref base) = base_dir {
        Path::new(base).join(&path)
    } else {
        // Relative with no base — resolve against home
        dirs::home_dir()
            .unwrap_or_else(|| Path::new(".").to_path_buf())
            .join(&path)
    };
    // Normalize the path (resolve .. and .)
    Ok(path_to_string(&normalize_path(&resolved)))
}

/// Normalize a path by resolving `.` and `..` components without requiring
/// the path to exist on disk (unlike `canonicalize()`).
fn normalize_path(path: &Path) -> std::path::PathBuf {
    use std::path::Component;
    let mut result = std::path::PathBuf::new();
    for component in path.components() {
        match component {
            Component::ParentDir => { result.pop(); }
            Component::CurDir => {}
            other => result.push(other),
        }
    }
    result
}

/// Return the parent directory of a file path.
#[tauri::command]
pub fn fs_parent_dir(path: String) -> Result<String, String> {
    let p = Path::new(&path);
    match p.parent() {
        Some(parent) => Ok(path_to_string(parent)),
        None => Err("Path has no parent directory".to_string()),
    }
}

#[tauri::command]
pub fn fs_get_default_dirs() -> Result<DefaultDirs, String> {
    let home = dirs::home_dir()
        .map(|p| path_to_string(&p))
        .unwrap_or_default();
    let documents = dirs::document_dir()
        .map(|p| path_to_string(&p))
        .unwrap_or_default();
    let downloads = dirs::download_dir()
        .map(|p| path_to_string(&p))
        .unwrap_or_default();
    let desktop = dirs::desktop_dir()
        .map(|p| path_to_string(&p))
        .unwrap_or_default();

    Ok(DefaultDirs {
        home,
        documents,
        downloads,
        desktop,
    })
}

#[tauri::command]
pub fn fs_copy_file(src: String, dest: String) -> Result<(), String> {
    fs::copy(&src, &dest)
        .map(|_| ())
        .map_err(|e| format!("Failed to copy file: {}", e))
}

fn copy_dir_recursive(src: &Path, dest: &Path) -> Result<(), String> {
    fs::create_dir_all(dest).map_err(|e| format!("Failed to create dir {}: {}", dest.display(), e))?;
    for entry in fs::read_dir(src).map_err(|e| format!("Failed to read dir: {}", e))? {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let entry_path = entry.path();
        let dest_path = dest.join(entry.file_name());
        if entry_path.is_dir() {
            copy_dir_recursive(&entry_path, &dest_path)?;
        } else {
            fs::copy(&entry_path, &dest_path)
                .map_err(|e| format!("Failed to copy {}: {}", entry_path.display(), e))?;
        }
    }
    Ok(())
}

#[tauri::command]
pub fn fs_copy_dir(src: String, dest: String) -> Result<(), String> {
    copy_dir_recursive(Path::new(&src), Path::new(&dest))
}

#[tauri::command]
pub fn fs_stat_extended(path: String) -> Result<FileStatExtended, String> {
    let p = Path::new(&path);
    let symlink_meta = fs::symlink_metadata(p)
        .map_err(|e| format!("Failed to stat {}: {}", p.display(), e))?;
    let is_symlink = symlink_meta.file_type().is_symlink();

    let meta = fs::metadata(p)
        .map_err(|e| format!("Failed to stat {}: {}", p.display(), e))?;

    let modified = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let created = meta
        .created()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let name = p
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    let extension = p
        .extension()
        .map(|e| e.to_string_lossy().to_string())
        .unwrap_or_default();
    let readonly = meta.permissions().readonly();

    Ok(FileStatExtended {
        name,
        path: path_to_string(p),
        is_dir: meta.is_dir(),
        size: meta.len(),
        modified,
        created,
        extension,
        readonly,
        is_symlink,
    })
}

#[tauri::command]
pub fn fs_open_in_terminal(path: String) -> Result<(), String> {
    let p = Path::new(&path);
    let dir = if p.is_dir() { p } else { p.parent().unwrap_or(p) };
    let dir_str = dir.to_string_lossy().to_string();

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/c", "start", "cmd", "/k", &format!("cd /d \"{}\"", dir_str)])
            .spawn()
            .map_err(|e| format!("Failed to open terminal: {}", e))?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .args(["-a", "Terminal", &dir_str])
            .spawn()
            .map_err(|e| format!("Failed to open terminal: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        let terminals = [
            ("x-terminal-emulator", vec!["--working-directory", &dir_str]),
            ("gnome-terminal", vec!["--working-directory", &dir_str]),
            ("konsole", vec!["--workdir", &dir_str]),
            ("xfce4-terminal", vec!["--working-directory", &dir_str]),
            ("xterm", vec!["-e", &format!("cd '{}' && $SHELL", dir_str)]),
        ];
        let mut launched = false;
        for (cmd, args) in &terminals {
            if std::process::Command::new(cmd)
                .args(args)
                .spawn()
                .is_ok()
            {
                launched = true;
                break;
            }
        }
        if !launched {
            return Err("No terminal emulator found".to_string());
        }
    }

    Ok(())
}
