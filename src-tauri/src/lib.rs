mod download;
mod paths;

use std::sync::{Arc, Mutex};
use serde::Serialize;
// Emitter is the tauri trait that gives AppHandle its .emit() method
// State is how tauri injects shared app state into command handlers
use tauri::{AppHandle, Emitter, State};
use tokio::io::{AsyncBufReadExt, BufReader};
// Child represents the running yt-dlp process, i store it so i can kill it on cancel
use tokio::process::Child;

// the shape of every event i send to the frontend over the download://progress channel
// Serialize lets tauri convert this to JSON automatically before sending
#[derive(Clone, Serialize)]
struct ProgressPayload {
    event: String,
    percent: Option<f32>,
    message: Option<String>,
}

// From is a standard Rust trait for type conversions
// implementing it here means i can write ProgressPayload::from(event) anywhere in this file
// it also keeps the match logic out of the streaming loop, which would be noisy
impl From<download::ProgressEvent> for ProgressPayload {
    fn from(event: download::ProgressEvent) -> Self {
        match event {
            download::ProgressEvent::Downloading { percent } => Self {
                event: "downloading".to_string(),
                percent: Some(percent),
                message: None,
            },
            download::ProgressEvent::Converting => Self {
                event: "converting".to_string(),
                percent: None,
                message: None,
            },
            download::ProgressEvent::Error(msg) => Self {
                event: "error".to_string(),
                percent: None,
                message: Some(msg),
            },
        }
    }
}

// Arc lets me share ownership of the child between the command handler and the streaming task
// Mutex ensures only one of them can access it at a time
// Option because there is no child when no download is running
struct DownloadState(Arc<Mutex<Option<Child>>>);

// Default is a standard Rust trait for "give me a zero value of this type"
// tauri's .manage() needs it to initialise the state when the app starts
impl Default for DownloadState {
    fn default() -> Self {
        Self(Arc::new(Mutex::new(None)))
    }
}

// #[tauri::command] exposes this function to the frontend via invoke("start_download", {...})
// async because spawning a process and waiting on it are async operations
#[tauri::command]
async fn start_download(
    url: String,
    output_dir: String,
    app: AppHandle,
    state: State<'_, DownloadState>,
) -> Result<(), String> {
    // the inner block drops the lock before the await points below
    // holding a std Mutex lock across an await would deadlock
    {
        let guard = state.0.lock().map_err(|e| e.to_string())?;
        if guard.is_some() {
            return Err("a download is already in progress".to_string());
        }
    }

    download::validate_url(&url)?;

    let yt_dlp = paths::yt_dlp_path(&app)?;
    let ffmpeg = paths::ffmpeg_path(&app)?;
    let args = download::build_args(&url, std::path::Path::new(&output_dir), &ffmpeg);

    let mut child = tokio::process::Command::new(&yt_dlp)
        .args(&args)
        // stdout has nothing useful for us, yt-dlp writes progress to stderr
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("failed to spawn yt-dlp: {}", e))?;

    // i must take stderr before storing the child because storing it moves ownership
    // after the move i can no longer access child.stderr
    let stderr = child.stderr.take().ok_or("failed to capture stderr")?;

    // store the child so cancel_download can reach it later
    *state.0.lock().map_err(|e| e.to_string())? = Some(child);

    // clone the Arc and AppHandle so the spawned task can own them independently
    // Arc::clone just increments the reference count, it doesnt copy the data
    let child_arc = Arc::clone(&state.0);
    let app_clone = app.clone();

    // spawn runs this block concurrently without blocking the current command
    // the frontend gets its Ok(()) response immediately while the download runs in the background
    tokio::spawn(async move {
        let reader = BufReader::new(stderr);
        let mut lines = reader.lines();

        loop {
            match lines.next_line().await {
                Ok(Some(line)) => {
                    if let Some(event) = download::parse_progress(&line) {
                        let _ = app_clone.emit("download://progress", ProgressPayload::from(event));
                    }
                }
                Ok(None) => {
                    // EOF means the process has exited, either naturally or because it was killed
                    // i take the child out before awaiting wait() so i dont hold the Mutex across an await
                    let child = child_arc.lock().unwrap().take();

                    let payload = if let Some(mut c) = child {
                        match c.wait().await {
                            Ok(status) if status.success() => ProgressPayload {
                                event: "done".to_string(),
                                percent: Some(100.0),
                                message: None,
                            },
                            Ok(status) => ProgressPayload {
                                event: "error".to_string(),
                                percent: None,
                                message: Some(format!("yt-dlp exited with {}", status)),
                            },
                            Err(e) => ProgressPayload {
                                event: "error".to_string(),
                                percent: None,
                                message: Some(format!("failed to wait for process: {}", e)),
                            },
                        }
                    } else {
                        // child was already taken by cancel_download, so this was a cancellation
                        ProgressPayload {
                            event: "cancelled".to_string(),
                            percent: None,
                            message: None,
                        }
                    };

                    let _ = app_clone.emit("download://progress", payload);
                    break;
                }
                Err(e) => {
                    let _ = app_clone.emit(
                        "download://progress",
                        ProgressPayload {
                            event: "error".to_string(),
                            percent: None,
                            message: Some(format!("failed to read yt-dlp output: {}", e)),
                        },
                    );
                    *child_arc.lock().unwrap() = None;
                    break;
                }
            }
        }
    });

    Ok(())
}

#[tauri::command]
async fn cancel_download(state: State<'_, DownloadState>) -> Result<(), String> {
    // take the child out of state first so kill() runs outside the Mutex lock
    // kill() is async and holding a std Mutex across an await is not allowed
    let child = state.0.lock().map_err(|e| e.to_string())?.take();

    if let Some(mut c) = child {
        c.kill().await.map_err(|e| format!("failed to kill process: {}", e))?;
        Ok(())
    } else {
        Err("no download in progress".to_string())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        // manage registers DownloadState as shared state, tauri injects it into commands automatically
        .manage(DownloadState::default())
        .invoke_handler(tauri::generate_handler![start_download, cancel_download])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;
    use download::ProgressEvent;

    // i cant unit test the tauri commands directly without a full app context
    // but i can test the pure conversion logic and state initialisation

    #[test]
    fn download_state_starts_empty() {
        let state = DownloadState::default();
        assert!(state.0.lock().unwrap().is_none());
    }

    #[test]
    fn payload_from_downloading_carries_percent() {
        let payload = ProgressPayload::from(ProgressEvent::Downloading { percent: 75.0 });
        assert_eq!(payload.event, "downloading");
        assert_eq!(payload.percent, Some(75.0));
        assert!(payload.message.is_none());
    }

    #[test]
    fn payload_from_converting_has_no_percent() {
        let payload = ProgressPayload::from(ProgressEvent::Converting);
        assert_eq!(payload.event, "converting");
        assert!(payload.percent.is_none());
        assert!(payload.message.is_none());
    }

    #[test]
    fn payload_from_error_carries_message() {
        let payload = ProgressPayload::from(ProgressEvent::Error("something went wrong".to_string()));
        assert_eq!(payload.event, "error");
        assert_eq!(payload.message, Some("something went wrong".to_string()));
        assert!(payload.percent.is_none());
    }
}
