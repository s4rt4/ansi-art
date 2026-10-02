# Draws app-icon.png (terminal window with a smiley made of colored character cells).
# Usage: python make-icon.py app-icon.png preview.png && pnpm tauri icon src-tauri/app-icon.png
from PIL import Image, ImageDraw
import colorsys, math, sys

S = 1024
SS = 2  # supersample for smooth edges
W = S * SS
img = Image.new("RGBA", (W, W), (0, 0, 0, 0))
d = ImageDraw.Draw(img)
k = lambda v: int(v * SS)

# Terminal window
M = 56
d.rounded_rectangle((k(M), k(M), k(S - M), k(S - M)), k(150), fill=(20, 21, 30, 255))
d.rounded_rectangle((k(M), k(M), k(S - M), k(S - M)), k(150), outline=(58, 61, 82, 255), width=k(10))
# Title bar separator + traffic-light dots
bar = M + 130
d.line((k(M + 10), k(bar), k(S - M - 10), k(bar)), fill=(40, 42, 56, 255), width=k(6))
for i, col in enumerate([(255, 95, 87), (254, 188, 46), (40, 200, 64)]):
    cx, cy, r = M + 115 + i * 78, M + 68, 26
    d.ellipse((k(cx - r), k(cy - r), k(cx + r), k(cy + r)), fill=col + (255,))

# Smiley made of terminal "cells" (taller than wide, like a real character grid)
FACE = [
    "....#####....",
    "..#########..",
    ".###########.",
    ".###########.",
    "###..###..###",
    "###..###..###",
    "#############",
    "#############",
    "##.#######.##",
    ".##.#####.##.",
    ".###.....###.",
    "..#########..",
    "....#####....",
]
COLS, ROWS = len(FACE[0]), len(FACE)
cw = ch = 46
gap = 7
ox = S / 2 - COLS * cw / 2
oy = (bar + S - M) / 2 - ROWS * ch / 2
for gy, row in enumerate(FACE):
    for gx, c in enumerate(row):
        if c != "#":
            continue
        hue = ((gx + gy) / (COLS + ROWS - 2)) * 0.83
        rr, gg, bb = colorsys.hsv_to_rgb(hue, 0.78, 1.0)
        x0, y0 = ox + gx * cw, oy + gy * ch
        d.rounded_rectangle((k(x0 + gap / 2), k(y0 + gap / 2), k(x0 + cw - gap / 2), k(y0 + ch - gap / 2)),
                            k(8), fill=(int(rr * 255), int(gg * 255), int(bb * 255), 255))

img = img.resize((S, S), Image.LANCZOS)
img.save(sys.argv[1])
# small-size previews to judge legibility
prev = Image.new("RGBA", (S + 32 + 128 + 16 + 64 + 16 + 32, S), (240, 240, 240, 255))
prev.paste(img, (0, 0), img)
x = S + 32
for sz in (128, 64, 32):
    t = img.resize((sz, sz), Image.LANCZOS); prev.paste(t, (x, 40), t); x += sz + 16
prev.save(sys.argv[2])
