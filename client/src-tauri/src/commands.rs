use std::fs;
use std::io::{BufRead, BufReader};
use std::net::TcpStream;
use std::process::{Command, Stdio};
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter};

/// Read the access token from ~/.pocketpaw/access_token
#[tauri::command]
pub fn read_access_token() -> Result<String, String> {
    let home = dirs::home_dir().ok_or("Could not determine home directory")?;
    let token_path = home.join(".pocketpaw").join("access_token");

    fs::read_to_string(&token_path)
        .map(|s| s.trim().to_string())
        .map_err(|e| format!("Failed to read token: {}", e))
}

/// Return the PocketPaw config directory path
#[tauri::command]
pub fn get_pocketpaw_config_dir() -> Result<String, String> {
    let home = dirs::home_dir().ok_or("Could not determine home directory")?;
    let config_dir = home.join(".pocketpaw");
    Ok(config_dir.to_string_lossy().to_string())
}

/// Check if a backend is running on the given port
#[tauri::command]
pub fn check_backend_running(port: u16) -> Result<bool, String> {
    let addr = format!("127.0.0.1:{}", port);
    match TcpStream::connect_timeout(
        &addr.parse().map_err(|e| format!("Invalid address: {}", e))?,
        Duration::from_secs(2),
    ) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

#[derive(Serialize, Clone)]
pub struct InstallStatus {
    pub installed: bool,
    pub has_config_dir: bool,
    pub has_cli: bool,
    pub config_dir: String,
}

/// Check if PocketPaw is installed (config dir + CLI in PATH)
#[tauri::command]
pub fn check_pocketpaw_installed() -> Result<InstallStatus, String> {
    let home = dirs::home_dir().ok_or("Could not determine home directory")?;
    let config_dir = home.join(".pocketpaw");
    let has_config_dir = config_dir.is_dir();

    let has_cli = if cfg!(windows) {
        Command::new("where")
            .arg("pocketpaw")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    } else {
        Command::new("which")
            .arg("pocketpaw")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    };

    Ok(InstallStatus {
        installed: has_config_dir && has_cli,
        has_config_dir,
        has_cli,
        config_dir: config_dir.to_string_lossy().to_string(),
    })
}

#[derive(Serialize, Clone)]
pub struct InstallProgress {
    pub line: String,
    pub done: bool,
    pub success: bool,
}

/// Install PocketPaw by spawning a non-interactive installer process.
/// Streams stdout line-by-line via "install-progress" events.
#[tauri::command]
pub async fn install_pocketpaw(app: AppHandle, profile: String) -> Result<bool, String> {
    // Run the Python installer directly in non-interactive mode.
    // We avoid the wrapper scripts (install.ps1/install.sh) because they rely on
    // an interactive console ([Console]::OutputEncoding / Rich) which isn't
    // available when spawned headless from Tauri with piped stdout/stderr.
    //
    // Flow: download installer.py to temp dir, run with --non-interactive --profile.
    let child = if cfg!(windows) {
        // Single PowerShell command: download installer.py then run it non-interactively.
        let ps_cmd = format!(
            "$tmp = Join-Path $env:TEMP 'pocketpaw_installer.py'; \
             Invoke-WebRequest -Uri 'https://raw.githubusercontent.com/pocketpaw/pocketpaw/main/installer/installer.py' \
               -OutFile $tmp -UseBasicParsing; \
             python $tmp --non-interactive --profile {} --uv-available --no-launch; \
             Remove-Item $tmp -ErrorAction SilentlyContinue",
            profile
        );
        Command::new("powershell")
            .args([
                "-NonInteractive",
                "-ExecutionPolicy",
                "Bypass",
                "-Command",
                &ps_cmd,
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    } else {
        let sh_cmd = format!(
            "tmp=$(mktemp /tmp/pocketpaw_installer.XXXXXX.py) && \
             curl -fsSL https://raw.githubusercontent.com/pocketpaw/pocketpaw/main/installer/installer.py -o \"$tmp\" && \
             python3 \"$tmp\" --non-interactive --profile {} --uv-available --no-launch; \
             rm -f \"$tmp\"",
            profile
        );
        Command::new("sh")
            .args(["-c", &sh_cmd])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    };

    let mut child = child.map_err(|e| format!("Failed to spawn installer: {}", e))?;

    let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;
    let reader = BufReader::new(stdout);

    for line in reader.lines() {
        match line {
            Ok(text) => {
                let _ = app.emit(
                    "install-progress",
                    InstallProgress {
                        line: text,
                        done: false,
                        success: false,
                    },
                );
            }
            Err(_) => break,
        }
    }

    let status = child
        .wait()
        .map_err(|e| format!("Failed to wait for installer: {}", e))?;
    let success = status.success();

    let _ = app.emit(
        "install-progress",
        InstallProgress {
            line: if success {
                "Installation complete!".to_string()
            } else {
                "Installation failed.".to_string()
            },
            done: true,
            success,
        },
    );

    Ok(success)
}

/// Start the PocketPaw backend as a detached background process on the given port.
/// Returns immediately — frontend should poll check_backend_running to confirm.
#[tauri::command]
pub fn start_pocketpaw_backend(port: u16) -> Result<bool, String> {
    let port_str = port.to_string();

    let result = if cfg!(windows) {
        // Use PowerShell Start-Process -WindowStyle Hidden to launch completely hidden.
        // The .exe shims from uv tool install are console apps that flash a CMD window
        // even with CREATE_NO_WINDOW, so we route through PowerShell instead.
        let ps_cmd = format!(
            "Start-Process -FilePath 'pocketpaw' -ArgumentList 'serve','--port','{port}' -WindowStyle Hidden",
            port = port_str
        );
        Command::new("powershell")
            .args([
                "-NonInteractive",
                "-ExecutionPolicy",
                "Bypass",
                "-Command",
                &ps_cmd,
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .or_else(|_| {
                // Fallback: try uv run
                let ps_cmd_uv = format!(
                    "Start-Process -FilePath 'uv' -ArgumentList 'run','pocketpaw','serve','--port','{port}' -WindowStyle Hidden",
                    port = port_str
                );
                Command::new("powershell")
                    .args([
                        "-NonInteractive",
                        "-ExecutionPolicy",
                        "Bypass",
                        "-Command",
                        &ps_cmd_uv,
                    ])
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn()
            })
    } else {
        Command::new("pocketpaw")
            .args(["serve", "--port", &port_str])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .or_else(|_| {
                Command::new("uv")
                    .args(["run", "pocketpaw", "serve", "--port", &port_str])
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn()
            })
    };

    match result {
        Ok(_) => Ok(true),
        Err(e) => Err(format!("Failed to start backend: {}", e)),
    }
}
