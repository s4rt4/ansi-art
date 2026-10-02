"""Renders the README sample images.

Runs the `ansi-art` CLI with `--format json` for each sample, then paints the
cell grid to PNG the way a terminal would. Block, quadrant, sextant, shade and
braille characters are drawn as exact shapes so they tile without font gaps.

Usage (from the repo root, after `cargo build -p ansi-cli`):
    python docs/render-samples.py
"""

import json
import subprocess
import sys
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parent.parent
CLI = ROOT / "target" / "debug" / ("ansi-art.exe" if sys.platform == "win32" else "ansi-art")
FONT = ROOT / "crates" / "ansi-core" / "assets" / "JetBrainsMono-Regular.ttf"
OUT = ROOT / "docs" / "samples"

SCALE = 2
CW, CH = 10 * SCALE, 20 * SCALE
PAD = 16 * SCALE
BACKGROUND = (12, 12, 12)
FOREGROUND = (204, 204, 204)

LOGO = "samples/logo.png"
ICON = "app/src-tauri/app-icon.png"

# (file name, source image, CLI arguments)
SAMPLES = [
    ("glyph", LOGO, ["-m", "glyph", "-w", "40"]),
    ("glyph-letters", LOGO, ["-m", "glyph", "-w", "40", "--charset", "letters", "--fill", "c", "--glyph-colors", "2"]),
    ("half-block", ICON, ["-m", "half-block", "-w", "40"]),
    ("quadrant", LOGO, ["-m", "quadrant", "-w", "40"]),
    ("sextant", LOGO, ["-m", "sextant", "-w", "40"]),
    ("braille", LOGO, ["-m", "braille", "-w", "40", "--threshold", "0.05"]),
    ("ascii", LOGO, ["-m", "ascii", "-w", "40", "--ramp", "detailed"]),
    ("ansi16-dither", LOGO, ["-m", "half-block", "-w", "40", "-c", "ansi16", "-d", "floyd-steinberg"]),
]

QUADRANTS = {"▘": 1, "▝": 2, "▀": 3, "▖": 4, "▌": 5, "▞": 6, "▛": 7, "▗": 8,
             "▚": 9, "▐": 10, "▜": 11, "▄": 12, "▙": 13, "▟": 14, "█": 15}
SHADES = {"░": 0.25, "▒": 0.5, "▓": 0.75}
BRAILLE_DOTS = [(0x01, 0, 0), (0x02, 0, 1), (0x04, 0, 2), (0x08, 1, 0),
                (0x10, 1, 1), (0x20, 1, 2), (0x40, 0, 3), (0x80, 1, 3)]
# Quantized-palette RGB for the 16-color sample, mirroring color::XTERM16.
XTERM16 = [(0, 0, 0), (205, 0, 0), (0, 205, 0), (205, 205, 0), (0, 0, 238), (205, 0, 205),
           (0, 205, 205), (229, 229, 229), (127, 127, 127), (255, 0, 0), (0, 255, 0),
           (255, 255, 0), (92, 92, 255), (255, 0, 255), (0, 255, 255), (255, 255, 255)]


def sextant_bits(cp: int) -> int:
    n = cp - 0x1FB00 + 1
    if n >= 21:
        n += 1
    if n >= 42:
        n += 1
    return n


def nearest16(c):
    return min(XTERM16, key=lambda p: sum((a - b) ** 2 for a, b in zip(p, c)))


def paint(grid, path: Path, palette=None):
    cols, rows = grid["cols"], grid["rows"]
    # Crop empty rows/columns so every sample is framed tightly.
    used = [(i % cols, i // cols) for i, c in enumerate(grid["cells"]) if c["ch"] != " " or c["bg"]]
    x0, x1 = min(x for x, _ in used), max(x for x, _ in used)
    y0, y1 = min(y for _, y in used), max(y for _, y in used)

    img = Image.new("RGB", ((x1 - x0 + 1) * CW + 2 * PAD, (y1 - y0 + 1) * CH + 2 * PAD), BACKGROUND)
    d = ImageDraw.Draw(img)
    font = ImageFont.truetype(str(FONT), int(15 * SCALE))
    ascent, descent = font.getmetrics()
    baseline = (CH - (ascent + descent)) // 2 + ascent
    quant = palette or (lambda c: tuple(c))

    for i, cell in enumerate(grid["cells"]):
        cx, cy = i % cols, i // cols
        if not (x0 <= cx <= x1 and y0 <= cy <= y1):
            continue
        X, Y = PAD + (cx - x0) * CW, PAD + (cy - y0) * CH
        ch = cell["ch"]
        fg = quant(cell["fg"]) if cell["fg"] else FOREGROUND
        if cell["bg"]:
            d.rectangle((X, Y, X + CW - 1, Y + CH - 1), fill=quant(cell["bg"]))
        if ch == " ":
            continue
        cp = ord(ch)
        rect = lambda gx, gy, nx, ny: d.rectangle(
            (X + gx * CW // nx, Y + gy * CH // ny, X + (gx + 1) * CW // nx - 1, Y + (gy + 1) * CH // ny - 1), fill=fg)
        if ch in QUADRANTS:
            bits = QUADRANTS[ch]
            for b, (gx, gy) in enumerate([(0, 0), (1, 0), (0, 1), (1, 1)]):
                if bits & (1 << b):
                    rect(gx, gy, 2, 2)
        elif 0x1FB00 <= cp <= 0x1FB3B:
            bits = sextant_bits(cp)
            for b in range(6):
                if bits & (1 << b):
                    rect(b % 2, b // 2, 2, 3)
        elif ch in SHADES:
            base = quant(cell["bg"]) if cell["bg"] else BACKGROUND
            a = SHADES[ch]
            mix = tuple(int(f * a + b * (1 - a)) for f, b in zip(fg, base))
            d.rectangle((X, Y, X + CW - 1, Y + CH - 1), fill=mix)
        elif 0x2800 <= cp <= 0x28FF:
            r = CW * 0.16
            for bit, gx, gy in BRAILLE_DOTS:
                if (cp - 0x2800) & bit:
                    px, py = X + (gx + 0.5) * CW / 2, Y + (gy + 0.5) * CH / 4
                    d.ellipse((px - r, py - r, px + r, py + r), fill=fg)
        else:
            d.text((X + CW / 2, Y + baseline), ch, font=font, fill=fg, anchor="ms")
    img.save(path, optimize=True)


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    for name, src, args in SAMPLES:
        out = subprocess.run([str(CLI), src, "-f", "json", *args], cwd=ROOT, capture_output=True, check=True)
        grid = json.loads(out.stdout)
        palette = nearest16 if "ansi16" in args else None
        paint(grid, OUT / f"{name}.png", palette)
        print(f"docs/samples/{name}.png  ({grid['cols']}x{grid['rows']})")


if __name__ == "__main__":
    main()
