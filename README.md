# MPGrab

MPGrab is a YouTube to MP3 downloader built with Tauri v2, React, and TypeScript.

## Features

- Single executable, no external dependencies required.
- Windows and Linux support.
- Real-time download progress.
- Custom output folder selection.

> See the [Changelog](CHANGELOG.md) for release history and upcoming features.

## Design Decisions

**Tauri over Electron.** Electron ships with its own Chromium instance, which adds around 150MB to the binary instantly. Tauri uses the OS WebView, so the final binary stays under 15MB. This matters for a minimalist tool like MPGrab.

**Rust for the backend.** The backend does process spawning, file I/O, and binary extraction. Rust is the right tool for these. The frontend stays in TypeScript, so Rust only appears where it makes sense.

**Embedded yt-dlp and ffmpeg.** I consider installing external tools a real friction point for the end user. Both binaries ship inside the executable and get extracted to the system temp directory on first run. No setup required.

**Automated releases.** Every release triggers a GitHub Actions pipeline that builds binaries for Linux and Windows. Versioning is derived from commit history automatically.

## Technical Implementation

- **Tauri v2:** Desktop shell backed by Rust. Uses the system WebView, so no browser engine is bundled into the binary.
- **React + TypeScript:** All UI logic lives in the frontend, decoupled from the backend.
- **IPC:** The frontend calls Rust backend functions via Tauri commands. Downloads run on a background thread and stream progress to the UI via Tauri events.
- **Embedded binaries:** `yt-dlp` and `ffmpeg` are bundled inside the executable and extracted to the system temp directory on first run.

## Installation

### Windows

1. Download `mpgrab.exe` from the [Releases](../../releases) page.
2. Double-click to run.

> Windows Defender may show a warning because the app is unsigned. Click **More info** then **Run anyway**.

### Linux

1. Download `mpgrab` from the [Releases](../../releases) page.
2. Make it executable and run:

```bash
chmod +x mpgrab
./mpgrab
```
