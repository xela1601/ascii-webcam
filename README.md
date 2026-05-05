# Ascii-Terminal-Webcam

A terminal-based webcam that renders your live feed as ASCII art, using OpenCV and Asciimatics.

![](https://github.com/sshlpe/Ascii-Terminal-Webcam/blob/main/assets/rezise_example.gif)

## Requirements

- [uv](https://docs.astral.sh/uv/) (handles Python and dependencies)

## Install

```
git clone https://github.com/sshlpe/Ascii-Terminal-Webcam.git
cd Ascii-Terminal-Webcam
uv sync
```

## Use

```
python webcam.py
```

To start with inverted brightness (can also toggle by pressing `r` at any time):

```
python webcam.py -r
```

## Controls

| Key | Action |
|-----|--------|
| `q` | Quit |
| `r` | Invert brightness |
| `c` | Save a capture to the `captures/` folder |

## Notes

- The webcam auto-adjusts to the terminal size, even when resizing after the program starts.
- Captures are stored in the `captures/` folder and overwritten when you run the program again. Move them elsewhere if you want to keep them.
- FPS depends on terminal size.
