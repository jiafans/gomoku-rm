#!/usr/bin/env python3
"""Generate gomoku-rm AppLoad icon: a 9x9 board fragment with two stones.

Pure-PIL output → packaging/icon.png. No external assets.
"""
from PIL import Image, ImageDraw

SIZE = 256
MARGIN = 24
GRID = 8  # 9 lines = 8 spaces

img = Image.new("RGB", (SIZE, SIZE), "white")
d = ImageDraw.Draw(img)

# Outer frame (subtle)
d.rectangle([0, 0, SIZE - 1, SIZE - 1], outline="black", width=2)

# Grid
inner = SIZE - 2 * MARGIN
spacing = inner / GRID
def gx(c: int) -> float: return MARGIN + c * spacing
def gy(r: int) -> float: return MARGIN + r * spacing

for i in range(GRID + 1):
    d.line([(gx(0), gy(i)), (gx(GRID), gy(i))], fill="black", width=2)
    d.line([(gx(i), gy(0)), (gx(i), gy(GRID))], fill="black", width=2)

# Star points: 4 corners + center of a 9-line grid
for c, r in [(2, 2), (2, 6), (6, 2), (6, 6), (4, 4)]:
    cx, cy = gx(c), gy(r)
    rad = 3
    d.ellipse([cx - rad, cy - rad, cx + rad, cy + rad], fill="black")

# Two stones — black at (3,4), white at (4,4) — a plausible mid-game snapshot
stone_r = int(spacing * 0.45)

bx, by = gx(3), gy(4)
d.ellipse([bx - stone_r, by - stone_r, bx + stone_r, by + stone_r], fill="black")

wx, wy = gx(5), gy(4)
d.ellipse([wx - stone_r, wy - stone_r, wx + stone_r, wy + stone_r],
          fill="white", outline="black", width=3)

# A diagonal black + diagonal white to show "in play" vibe
bx2, by2 = gx(4), gy(3)
d.ellipse([bx2 - stone_r, by2 - stone_r, bx2 + stone_r, by2 + stone_r], fill="black")

wx2, wy2 = gx(4), gy(5)
d.ellipse([wx2 - stone_r, wy2 - stone_r, wx2 + stone_r, wy2 + stone_r],
          fill="white", outline="black", width=3)

img.save("packaging/icon.png", "PNG", optimize=True)
print(f"wrote packaging/icon.png ({SIZE}x{SIZE})")
