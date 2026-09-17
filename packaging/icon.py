#!/usr/bin/env python3
"""Ensure the 1024px app icon PNG exists for installers and .icns generation."""

from __future__ import annotations

import math
import struct
import zlib
from pathlib import Path


def mix(a: float, b: float, t: float) -> float:
    return a + (b - a) * t


def smoothstep(edge0: float, edge1: float, x: float) -> float:
    t = max(0.0, min(1.0, (x - edge0) / (edge1 - edge0)))
    return t * t * (3.0 - 2.0 * t)


def sd_round_box(px: float, py: float, hx: float, hy: float, radius: float) -> float:
    ax = abs(px) - hx + radius
    ay = abs(py) - hy + radius
    ox = max(ax, 0.0)
    oy = max(ay, 0.0)
    return math.hypot(ox, oy) + min(max(ax, ay), 0.0) - radius


def pixel(x: int, y: int, size: int) -> tuple[int, int, int, int]:
    nx = (x + 0.5) / size
    ny = (y + 0.5) / size
    sx = ((nx - 0.5) * 2.0) / 0.90
    sy = ((ny - 0.5) * 2.0) / 0.90
    squircle = abs(sx) ** 5.0 + abs(sy) ** 5.0
    alpha = smoothstep(1.03, 0.97, squircle)
    if alpha <= 0.0:
        return (0, 0, 0, 0)

    glow = math.exp(-((nx - 0.5) ** 2) * 6.0 - ((ny - 0.18) ** 2) * 10.0)
    r = mix(6, 28, glow)
    g = mix(14, 48, glow)
    b = mix(12, 28, glow)

    rim = smoothstep(0.90, 0.98, squircle) * smoothstep(1.02, 0.99, squircle)
    r = mix(r, 40, rim)
    g = mix(g, 230, rim)
    b = mix(b, 110, rim)

    gx = nx - 0.50
    gy = ny - 0.42
    radius = math.hypot(gx, gy)
    angle = math.atan2(gy, gx)
    # Gauge arc: 210 degrees, opening at the bottom.
    start, end = math.radians(155), math.radians(385)
    wrapped = angle if angle >= 0 else angle + 2.0 * math.pi
    on_arc = start <= wrapped <= end or start <= wrapped + 2.0 * math.pi <= end
    ring = abs(radius - 0.22)
    if on_arc and ring < 0.034:
        t = 1.0 - ring / 0.034
        r = mix(r, 26, t)
        g = mix(g, 255, t)
        b = mix(b, 96, t)

    bars = [(0.38, 0.10), (0.46, 0.14), (0.54, 0.18), (0.62, 0.22)]
    for left, height in bars:
        cx = left - 0.50
        cy = 0.78 - height / 2.0 - 0.50
        d = sd_round_box((nx - 0.5) - cx, (ny - 0.5) - cy, 0.028, height / 2.0, 0.012)
        if d < 0.0:
            t = smoothstep(0.008, -0.004, d)
            r = mix(r, 26, t)
            g = mix(g, 255, t)
            b = mix(b, 96, t)

    return (int(r), int(g), int(b), int(round(alpha * 255)))


def write_png(path: Path, size: int = 1024) -> None:
    raw = bytearray()
    for y in range(size):
        raw.append(0)
        for x in range(size):
            raw.extend(pixel(x, y, size))
    ihdr = struct.pack(">IIBBBBB", size, size, 8, 6, 0, 0, 0)

    def chunk(tag: bytes, payload: bytes) -> bytes:
        return (
            struct.pack(">I", len(payload))
            + tag
            + payload
            + struct.pack(">I", zlib.crc32(tag + payload) & 0xFFFFFFFF)
        )

    png = b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", ihdr) + chunk(b"IDAT", zlib.compress(bytes(raw), 9)) + chunk(b"IEND", b"")
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(png)


def main() -> None:
    dest = Path(__file__).resolve().parent / "desk-monitoring.png"
    if dest.is_file() and dest.stat().st_size > 1024:
        print(dest)
        return
    write_png(dest)
    print(dest)


if __name__ == "__main__":
    main()
