# Changelog

All notable changes to this project will be documented in this file.

## [1.0.0] - 2026-05-05

### Added
- Rust rewrite of the original Python ASCII webcam (https://github.com/sshlpe/Ascii-Terminal-Webcam) for maximum performance
- 24-bit true-color ANSI output (each pixel retains its original RGB color)
- Synchronized terminal output to eliminate flickering
- HUD status bar with FPS counter and control hints
- Stretch/Fit scaling toggle (`f` key)
- Horizontal mirror toggle (`-m` flag / `m` key)
- Auto-detect portrait camera orientation and rotate to landscape
- Dynamic terminal resize support
- Homebrew formula for easy macOS installation (arm64 + x86_64)
- mise tool configuration for reproducible Rust toolchain

### Changed
- Rewrote from Python (OpenCV/PIL) to Rust (nokhwa/crossterm)
- Precomputed 256-entry grayscale lookup table for O(1) pixel mapping
- Nearest-neighbour sampling with center-cropped aspect ratio handling
- Single-buffer stdout writes per frame

### Removed
- Python runtime and OpenCV system dependencies
- `pyfiglet`, `future`, `wcwidth` transitive dependencies

[1.0.0]: https://github.com/xela1601/ascii-webcam/releases/tag/v1.0.0
