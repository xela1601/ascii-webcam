# Changelog

All notable changes to this project will be documented in this file.

## [1.1.0] - 2026-05-07

### Added
- Character cell aspect-ratio compensation — camera feed no longer appears vertically stretched
- `-t` / `--transpose` flag and `t` key to swap raw buffer width/height
- Camera resolution displayed in HUD (`160x120 -> 92x69`)

### Changed
- Removed fill/stretch mode — always fit-within, preserving aspect ratio and showing the full frame
- Simplified camera open to use `RequestedFormatType::None` (avoids macOS `AbsoluteHighestResolution` panic)
- Reordered format strategy fallback for better error recovery

### Fixed
- Raw buffer transpose for portrait-oriented camera sensors
- apt sources.list format to use `stable main` distribution

## [1.0.1] - 2026-05-07

### Added
- Human-readable camera error messages: detects missing hardware vs. permission denial
- Linux arm64 `.deb` package (Raspberry Pi support)
- proper apt repository on GitHub Pages with `stable main` distribution

### Fixed
- apt sources.list format (`stable main` instead of flat `./`)
- aarch64-linux-gnu cross-compilation linker (`aarch64-linux-gnu-gcc`)
- `catch_unwind` wrapper around `Camera::new` to prevent macOS permission panics
- camera open retry across three format strategies on failure

## [1.0.0] - 2026-05-05

### Added
- Rust rewrite of the original Python ASCII webcam (https://github.com/sshlpe/Ascii-Terminal-Webcam) for maximum performance
- 24-bit true-color ANSI output (each pixel retains its original RGB color)
- Synchronized terminal output to eliminate flickering
- HUD status bar with FPS counter and control hints
- Stretch/Fit scaling toggle (`f` key)
- Horizontal mirror toggle (`-m` flag / `m` key)
- Dynamic terminal resize support
- Homebrew formula for easy macOS installation (arm64 + x86_64)
- apt repository for Debian/Ubuntu installation (amd64 + arm64)
- mise tool configuration for reproducible Rust toolchain
- GitHub Actions release workflow with changelog-driven releases

### Changed
- Rewrote from Python (OpenCV/PIL) to Rust (nokhwa/crossterm)
- Precomputed 256-entry grayscale lookup table for O(1) pixel mapping
- Nearest-neighbour sampling with center-cropped aspect ratio handling
- Single-buffer stdout writes per frame

### Removed
- Python runtime and OpenCV system dependencies
- `pyfiglet`, `future`, `wcwidth` transitive dependencies

[1.1.0]: https://github.com/xela1601/ascii-webcam/releases/tag/v1.1.0
[1.0.1]: https://github.com/xela1601/ascii-webcam/releases/tag/v1.0.1
[1.0.0]: https://github.com/xela1601/ascii-webcam/releases/tag/v1.0.0
