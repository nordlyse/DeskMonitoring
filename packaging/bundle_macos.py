#!/usr/bin/env python3
"""Bundle Desk Monitor as a macOS .app and .dmg, including Homebrew GTK dylibs."""

from __future__ import annotations

import os
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DIST = ROOT / "dist" / "macos"
APP = DIST / "Desk Monitor.app"
CONTENTS = APP / "Contents"
MACOS = CONTENTS / "MacOS"
FRAMEWORKS = CONTENTS / "Frameworks"
RESOURCES = CONTENTS / "Resources"
SKIP_PREFIXES = ("/usr/lib/", "/System/", "/Library/Apple/")


def run(cmd: list[str], **kwargs) -> subprocess.CompletedProcess[str]:
    return subprocess.run(cmd, check=True, text=True, **kwargs)


def binary_path() -> Path:
    env_dir = os.environ.get("CARGO_TARGET_DIR")
    if env_dir:
        candidate = Path(env_dir) / "release" / "desk-monitoring"
        if candidate.is_file():
            return candidate
    local = ROOT / "target" / "release" / "desk-monitoring"
    if local.is_file():
        return local
    raise SystemExit("Release binary not found. Run cargo build --release first.")


def load_dylibs(path: Path) -> list[str]:
    out = subprocess.check_output(["otool", "-L", str(path)], text=True)
    deps: list[str] = []
    for line in out.splitlines()[1:]:
        item = line.strip().split(" (", 1)[0]
        if not item or item.startswith("@") or item.startswith(SKIP_PREFIXES):
            continue
        deps.append(item)
    return deps


def resolve_real(path: str) -> Path | None:
    p = Path(path)
    if p.is_file():
        return p.resolve()
    return None


def collect_closure(entry: Path) -> dict[str, Path]:
    pending = [entry]
    seen_files: dict[str, Path] = {}
    while pending:
        current = pending.pop()
        for dep in load_dylibs(current):
            real = resolve_real(dep)
            if real is None or str(real) in seen_files:
                continue
            seen_files[str(real)] = real
            pending.append(real)
    return seen_files


def copy_and_relink(entry: Path) -> None:
    mapping: dict[str, str] = {}
    files = collect_closure(entry)
    for real in files.values():
        dest_name = real.name
        dest = FRAMEWORKS / dest_name
        shutil.copy2(real, dest)
        os.chmod(dest, 0o755)
        mapping[str(real)] = f"@executable_path/../Frameworks/{dest_name}"
        # Homebrew often stores the install name as /usr/local/opt/...
        for dep in load_dylibs(real):
            mapping.setdefault(dep, f"@executable_path/../Frameworks/{Path(dep).name}")

    def rewrite(target: Path) -> None:
        if target.parent == FRAMEWORKS:
            run(
                [
                    "install_name_tool",
                    "-id",
                    f"@executable_path/../Frameworks/{target.name}",
                    str(target),
                ]
            )
        out = subprocess.check_output(["otool", "-L", str(target)], text=True)
        lines = out.splitlines()[1:]
        for index, line in enumerate(lines):
            item = line.strip().split(" (", 1)[0]
            if index == 0 and target.suffix == ".dylib":
                continue
            new = mapping.get(item)
            if new is None:
                real = resolve_real(item)
                if real is not None:
                    new = mapping.get(str(real))
            if new and new != item:
                subprocess.run(
                    ["install_name_tool", "-change", item, new, str(target)],
                    check=False,
                )

    rewrite(MACOS / "desk-monitoring")
    for lib in FRAMEWORKS.glob("*.dylib"):
        rewrite(lib)


def add_wrapper() -> None:
    wrapper = MACOS / "DeskMonitor"
    wrapper.write_text(
        """#!/bin/bash
set -euo pipefail
DIR="$(cd "$(dirname "$0")" && pwd)"
export DYLD_LIBRARY_PATH="$DIR/../Frameworks${DYLD_LIBRARY_PATH:+:$DYLD_LIBRARY_PATH}"
export GDK_PIXBUF_MODULE_FILE="$DIR/../Resources/gdk-pixbuf-2.0/loaders.cache"
export GSETTINGS_SCHEMA_DIR="$DIR/../Resources/glib-2.0/schemas"
export XDG_DATA_DIRS="$DIR/../Resources/share${XDG_DATA_DIRS:+:$XDG_DATA_DIRS}"
exec "$DIR/desk-monitoring" "$@"
""",
        encoding="utf-8",
    )
    os.chmod(wrapper, 0o755)


def copy_gtk_data() -> None:
    brew = subprocess.check_output(["brew", "--prefix"], text=True).strip()
    prefix = Path(brew)
    schema_src = prefix / "share" / "glib-2.0" / "schemas"
    if schema_src.is_dir():
        dest = RESOURCES / "glib-2.0" / "schemas"
        dest.mkdir(parents=True, exist_ok=True)
        for item in schema_src.glob("*"):
            if item.is_file():
                shutil.copy2(item, dest / item.name)
    pixbuf_src = prefix / "lib" / "gdk-pixbuf-2.0"
    if pixbuf_src.is_dir():
        dest = RESOURCES / "gdk-pixbuf-2.0"
        if dest.exists():
            shutil.rmtree(dest)
        shutil.copytree(pixbuf_src, dest)
        cache = dest / "loaders.cache"
        loaders_dir = next(dest.glob("*/loaders"), None)
        if loaders_dir and shutil.which("gdk-pixbuf-query-loaders"):
            env = os.environ.copy()
            env["GDK_PIXBUF_MODULEDIR"] = str(loaders_dir)
            text = subprocess.check_output(["gdk-pixbuf-query-loaders"], env=env, text=True)
            cache.write_text(text, encoding="utf-8")


def add_icon() -> None:
    png = ROOT / "packaging" / "desk-monitoring.png"
    if not png.is_file():
        run([sys.executable, str(ROOT / "packaging" / "icon.py")])
    iconset = DIST / "AppIcon.iconset"
    if iconset.exists():
        shutil.rmtree(iconset)
    iconset.mkdir(parents=True)
    sizes = [
        (16, "icon_16x16.png"),
        (32, "icon_16x16@2x.png"),
        (32, "icon_32x32.png"),
        (64, "icon_32x32@2x.png"),
        (128, "icon_128x128.png"),
        (256, "icon_128x128@2x.png"),
        (256, "icon_256x256.png"),
        (512, "icon_256x256@2x.png"),
        (512, "icon_512x512.png"),
        (1024, "icon_512x512@2x.png"),
    ]
    for px, name in sizes:
        run(["sips", "-z", str(px), str(px), str(png), "--out", str(iconset / name)], stdout=subprocess.DEVNULL)
    run(["iconutil", "-c", "icns", str(iconset), "-o", str(RESOURCES / "AppIcon.icns")])


def add_dmg() -> Path:
    dmg = ROOT / "dist" / "DeskMonitor-macos.dmg"
    if dmg.exists():
        dmg.unlink()
    run(
        [
            "hdiutil",
            "create",
            "-volname",
            "Desk Monitor",
            "-srcfolder",
            str(APP),
            "-ov",
            "-format",
            "UDZO",
            str(dmg),
        ],
        stdout=subprocess.DEVNULL,
    )
    return dmg


def main() -> None:
    shutil.rmtree(DIST, ignore_errors=True)
    MACOS.mkdir(parents=True)
    FRAMEWORKS.mkdir(parents=True)
    RESOURCES.mkdir(parents=True)
    shutil.copy2(ROOT / "packaging" / "Info.plist", CONTENTS / "Info.plist")
    shutil.copy2(binary_path(), MACOS / "desk-monitoring")
    os.chmod(MACOS / "desk-monitoring", 0o755)
    add_wrapper()
    add_icon()
    copy_and_relink(MACOS / "desk-monitoring")
    copy_gtk_data()
    run(["codesign", "--force", "--deep", "--sign", "-", str(APP)])
    dmg = add_dmg()
    print(APP)
    print(dmg)


if __name__ == "__main__":
    main()
