# ascii-webcam

A terminal-based webcam that renders your live feed as colored ASCII art, written in Rust for maximum performance.

## Install

### Homebrew (macOS, arm64 and x86_64)

Add the tap, then install:

```
brew tap xela1601/tap
brew install ascii-webcam
```

> The tap formula is auto-updated on each releas

### apt (Debian / Ubuntu, amd64 and arm64)

```
echo "deb [trusted=yes] https://xela1601.github.io/ascii-webcam/apt stable main" | \
  sudo tee /etc/apt/sources.list.d/ascii-webcam.list
sudo apt update
sudo apt install ascii-webcam
```

### Cargo

```
cargo install --git https://github.com/xela1601/ascii-webcam.git
```

### From source

```
git clone https://github.com/xela1601/ascii-webcam.git
cd ascii-webcam
cargo build --release
```

### With mise

If you use [mise](https://mise.jdx.dev/), the `.mise.toml` bootstraps the correct Rust version:

```
mise install
mise run build
```

## Use

```
ascii-webcam
```

Or during development:

```
cargo run --release
```

| Flag | Description |
|------|-------------|
| `-r`, `--invert` | Start with inverted brightness |
| `-m`, `--mirror`  | Start with horizontal mirror |
| `-b`, `--bw`  | Start in black & white mode |
| `--http`      | Start HTTP MJPEG stream |
| `-p`, `--port`   | HTTP stream port (default 8080) |
| `--v4l2 <dev>`   | Write to v4l2loopback device (Linux) |

## Streaming / Virtual camera

### HTTP MJPEG (`--http`)

```
ascii-webcam --http
```

Open `http://localhost:8080` in a browser, or use it as a camera source:

**OBS**: Add a **Media Source**, uncheck "Local File", set Input to `http://localhost:8080`, click **Start Virtual Camera**.

If OBS refuses the MJPEG stream, use VLC as a bridge: **VLC Video Source** (in OBS) → add `http://localhost:8080`.

### v4l2loopback — Linux (`--v4l2`)

Creates a virtual camera visible in Zoom, Chrome, etc. Setup once:

```
sudo modprobe v4l2loopback devices=1 video_nr=2 card_label="ascii-webcam"
```

```
ascii-webcam --v4l2 /dev/video2
```

The device appears as "ascii-webcam" in camera selection dialogs. Toggle with `s` key while running.

## Controls

| Key | Action |
|-----|--------|
| `q`, `Ctrl+C` | Quit |
| `r` | Toggle brightness invert |
| `m` | Toggle horizontal mirror |
| `c` | Save capture to `captures/` |
| `s` | Toggle v4l2 virtual camera output |

## Camera permissions

### macOS

macOS requires apps to be granted camera access. If the webcam doesn't open, allow your terminal app in **System Settings > Privacy & Security > Camera** and restart the app. If you're using VS Code's integrated terminal, VS Code itself needs the permission. If it's already listed and still fails, try:

```
tccutil reset Camera
```

### Linux

Ensure your user is in the `video` group:

```
sudo usermod -a -G video $USER
```

Log out and back in for the change to take effect.

## Notes

- The webcam auto-adjusts to the terminal size, even when resizing after the program starts.
- Captures are stored in the `captures/` folder and overwritten when you run the program again. Move them elsewhere if you want to keep them.
- FPS depends on terminal size.

## Homebrew tap

Is created automatically via workflow, when a semver tag is pushed.
The workflow will build both architectures, create the GitHub Release, and push the formula to https://github.com/xela1601/homebrew-tap.

```
git tag v1.0.0 && git push origin v1.0.0
```

## Releasing

Releases are driven by **semver tags** and **[git-cliff](https://git-cliff.org)** for changelogs.

1. Write [conventional commits](https://www.conventionalcommits.org) (`feat:`, `fix:`, `refactor:` etc.)
2. Bump the version in `Cargo.toml`
3. Tag and push:
   ```
   git tag v1.3.0 && git push origin v1.3.0
   ```
4. The workflow builds all architectures, generates the changelog with `git-cliff`, and publishes the release
