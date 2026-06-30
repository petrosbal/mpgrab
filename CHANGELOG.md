# Changelog

All notable changes to this project will be documented in this file.
The format is loosely based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [0.2.0](https://github.com/petrosbal/mpgrab/compare/mpgrab-v0.1.1...mpgrab-v0.2.0) (2026-06-30)


### Features

* add Tailwind CSS v4 and initialize shadcn/ui with Vega preset ([9bb56c2](https://github.com/petrosbal/mpgrab/commit/9bb56c2be984a55844aecd49c8b4883e04d43499))
* **backend:** add download logic, progress parsing, and Tauri commands ([4b1f6b3](https://github.com/petrosbal/mpgrab/commit/4b1f6b3456c7be0b4a36fe87d8acf0d99eaf7681))
* **frontend:** add initial download UI with progress and cancel support ([9749ee3](https://github.com/petrosbal/mpgrab/commit/9749ee3d8f0b2c78d87058c21327e6099d4fac6c))


### Bug Fixes

* **config:** remove deprecated baseUrl from tsconfig ([487d262](https://github.com/petrosbal/mpgrab/commit/487d2628f126f84507db0c9ffe4f3a1d45af7134))

## [Unreleased]

- **Real-time Progress Bar:** Visual percentage tracking for active downloads.
- **Direct Youtube Search and Discovery:** Search query support to retrieve and list top YouTube results, eliminating URL dependence.
- **FFmpeg Independency:** Native Rust audio conversion to eliminate FFmpeg dependency.
- **Audio Quality Selector:** Toggle between 128kbps, 192kbps, and 320kbps.
- **Metadata & ID3 Tags:** Automatic embedding of Title, Artist, and Album Art.
- **Playlist Support:** Sequential processing of entire YouTube playlists.
- **Input Validation:** URL cleaning and error handling.
- **Auto-Updater:** Automated update checks for the application and downloader engine.

## [0.1.1] - 2026-01-19

### Added

- GNU General Public License v3.0.
- Custom output folder selection using native dialogs.
- Source code split into multiple files for modularity.
- MpGrab logo inside the application.
- Icon for the application window and executable.
- Makefile Windows target for easier building on Windows systems.

## [0.1.0] - 2026-01-19

- Initial release.
- Minimal GUI with input field and download button.
- Threads for concurrent downloading and UI updates.
- Binary embedding for easy installation.
- Automated CI/CD pipelines for multi-platform support (Windows and Linux releases).
