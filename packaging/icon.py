#!/usr/bin/env python3
"""Write a 512x512 PNG icon for Desk Monitor."""

from __future__ import annotations

import struct
import zlib
from pathlib import Path


def pixel(x: int, y: int, size: int) -> tuple[int, int, int]:
    nx = (x + 0.5) / size
    ny = (y + 0.5) / size
    cx, cy = nx - 0.5, ny - 0.5
    r = (cx * cx + cy * cy) ** 0.5
    if r > 0.46:
        return (0, 0, 0)
    if r > 0.42:
        return (20, 90, 40)
    # dark panel
    g = 12 + int(40 * (1.0 - ny))
    rgb = (4, g, 14)
    # inner neon ring
    if 0.28 < r < 0.33:
        return (26, 255, 96)
    # grid
    if abs((x % 32) - 0) < 1 or abs((y % 32) - 0) < 1:
        return (18, min(255, g + 70), 36)
    # bar chart blocks
    bars = [(0.30, 0.62, 0.22), (0.42, 0.54, 0.34), (0.54, 0.66, 0.18), (0.66, 0.50, 0.40)]
    for left, top, height in bars:
        if left <= nx <= left + 0.08 and (0.70 - height) <= ny <= 0.70:
            return (26, 255, 96)
    return rgb


def write_png(path: Path, size: int = 512) -> None:
    raw = bytearray()
    for y in range(size):
        raw.append(0)
        for x in range(size):
            r, g, b = pixel(x, y, size)
            raw.extend((r, g, b))
    compressed = zlib.compress(bytes(raw), 9)
    ihdr = struct.pack(">IIBBBBB", size, size, 8, 2, 0, 0, 0)

    def chunk(tag: bytes, data: bytes) -> bytes:
        return (
            struct.pack(">I", len(data))
            + tag
            + data
            + struct.pack(">I", zlib.crc32(tag + data) & 0xFFFFFFFF)
        )

    png = b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", ihdr) + chunk(b"IDAT", compressed) + chunk(b"IEND", b"")
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(png)


def main() -> None:
    root = Path(__file__).resolve().parent
    write_png(root / "desk-monitoring.png")
    print(root / "desk-monitoring.png")


if __name__ == "__main__":
    main()
