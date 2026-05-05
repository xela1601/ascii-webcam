import cv2
import numpy as np
import sys
import time
from asciimatics.screen import Screen
from PIL import Image


def ascii_img(img, scale, reverse=False):
    gscale = "   .:-=+*#%@"

    if reverse:
        gscale = gscale[::-1]

    h, w = scale
    img = img.resize((w, h)).convert("L")
    arr_img = np.array(img)

    max_ = np.max(arr_img)
    min_ = np.min(arr_img)

    values = np.linspace(min_, max_, len(gscale))

    ascii_rows = []
    for row in arr_img:
        row_chars = ""
        for col in row:
            c = 0
            while col > values[c]:
                c += 1
            row_chars += gscale[c]
        ascii_rows.append(row_chars)

    return ascii_rows


def handle_input(screen, ascii_, capture_index):
    global reverse

    ev = screen.get_key()
    if ev == ord("q"):
        sys.exit()
    if ev == ord("r"):
        reverse = not reverse
    if ev == ord("c"):
        with open(f"captures/capture_{capture_index}.txt", "w") as file:
            file.write("\n".join(ascii_))
        capture_index += 1
    return capture_index


def demo(screen):
    start = time.time()
    frames = 0
    capture_index = 0

    while True:
        ret, frame = vid.read()
        frame = cv2.flip(frame, 1)
        frame = cv2.cvtColor(frame, cv2.COLOR_BGR2RGB)
        img = Image.fromarray(frame)

        ascii_ = ascii_img(img, screen.dimensions, reverse)

        for i, row in enumerate(ascii_):
            for u, char in enumerate(row):
                screen.print_at(char, u, i)

        capture_index = handle_input(screen, ascii_, capture_index)

        frames += 1
        elapsed = time.time() - start
        fps = int(frames / elapsed) if elapsed > 0 else 0
        screen.print_at(f"  {fps} FPS ", 0, 0)

        screen.refresh()

        if screen.has_resized():
            screen.clear()
            Screen.wrapper(demo)
            return

        time.sleep(0.0001)


reverse = False
if len(sys.argv) > 1 and sys.argv[1] == "-r":
    reverse = True

vid = cv2.VideoCapture(0)
Screen.wrapper(demo)


