#!/usr/bin/env python3
"""Bundle Desk Monitor as a macOS .app and .dmg, including Homebrew GTK dylibs."""

from __future__ import annotations

import os
import shutil
import subprocess
import sys
import time
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
    lines = out.splitlines()[1:]
    for index, line in enumerate(lines):
        item = line.strip().split(" (", 1)[0]
        if not item or item.startswith("@") or item.startswith(SKIP_PREFIXES):
            continue
        if index == 0 and path.suffix == ".dylib":
            continue
        deps.append(item)
    return deps


def resolve_real(path: str) -> Path | None:
    p = Path(path)
    if p.is_file() or p.is_symlink():
        return p.resolve()
    return None


def collect_aliases(entry: Path) -> dict[str, Path]:
    pending = [entry]
    seen_real: set[str] = set()
    aliases: dict[str, Path] = {}
    while pending:
        current = pending.pop()
        for dep in load_dylibs(current):
            real = resolve_real(dep)
            if real is None:
                continue
            aliases[Path(dep).name] = real
            aliases.setdefault(real.name, real)
            if str(real) not in seen_real:
                seen_real.add(str(real))
                pending.append(real)
    return aliases


def copy_and_relink(entry: Path) -> None:
    aliases = collect_aliases(entry)
    copied_primary: dict[str, str] = {}
    mapping: dict[str, str] = {}

    for dest_name, real in aliases.items():
        dest = FRAMEWORKS / dest_name
        bundled = f"@executable_path/../Frameworks/{dest_name}"
        mapping[str(real)] = bundled
        mapping.setdefault(dest_name, bundled)
        primary = copied_primary.get(str(real))
        if primary is None:
            shutil.copy2(real, dest)
            os.chmod(dest, 0o755)
            copied_primary[str(real)] = dest_name
        elif dest_name != primary and not dest.exists():
            os.symlink(primary, dest)

    for dest_name, real in aliases.items():
        mapping[f"@executable_path/../Frameworks/{real.name}"] = (
            f"@executable_path/../Frameworks/{dest_name}"
        )
        for dep in load_dylibs(real):
            mapping[dep] = f"@executable_path/../Frameworks/{Path(dep).name}"

    def rewrite(target: Path) -> None:
        if target.parent == FRAMEWORKS and not target.is_symlink():
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
                    new = mapping.get(str(real)) or mapping.get(real.name)
            if new is None:
                new = mapping.get(Path(item).name)
            if new and new != item:
                subprocess.run(
                    ["install_name_tool", "-change", item, new, str(target)],
                    check=False,
                )

    rewrite(entry)
    for lib in sorted(FRAMEWORKS.glob("*.dylib")):
        if lib.is_symlink():
            continue
        rewrite(lib)
    verify_links(entry)


def verify_links(entry: Path) -> None:
    missing: list[str] = []
    leftover: list[str] = []
    targets = [entry, *sorted(p for p in FRAMEWORKS.glob("*.dylib") if not p.is_symlink())]
    for target in targets:
        out = subprocess.check_output(["otool", "-L", str(target)], text=True)
        for line in out.splitlines()[1:]:
            item = line.strip().split(" (", 1)[0]
            if item.startswith("@executable_path/../Frameworks/"):
                name = item.rsplit("/", 1)[-1]
                if not (FRAMEWORKS / name).exists():
                    missing.append(f"{target.name} -> {name}")
            elif item.startswith(("/usr/local/", "/opt/homebrew/")):
                leftover.append(f"{target.name} -> {item}")
    if missing or leftover:
        details = "\n".join(missing + leftover)
        raise SystemExit(f"Bundled libraries are incomplete:\n{details}")


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
    run([sys.executable, str(ROOT / "packaging" / "icon.py")])
    if not png.is_file():
        raise SystemExit("missing packaging/desk-monitoring.png")
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
    rw = ROOT / "dist" / "DeskMonitor-macos.rw.dmg"
    stage = ROOT / "dist" / "dmg-root"
    mount = Path("/Volumes/Desk Monitor")
    shutil.rmtree(stage, ignore_errors=True)
    if dmg.exists():
        dmg.unlink()
    if rw.exists():
        rw.unlink()
    detach_desk_volumes()
    stage.mkdir(parents=True)
    shutil.copytree(APP, stage / APP.name, symlinks=True)
    os.symlink("/Applications", stage / "Applications")
    run(
        [
            "hdiutil",
            "create",
            "-volname",
            "Desk Monitor",
            "-srcfolder",
            str(stage),
            "-ov",
            "-fs",
            "HFS+",
            "-format",
            "UDRW",
            str(rw),
        ],
        stdout=subprocess.DEVNULL,
    )
    run(
        [
            "hdiutil",
            "attach",
            "-readwrite",
            "-noverify",
            "-noautoopen",
            "-mountpoint",
            str(mount),
            str(rw),
        ],
        stdout=subprocess.DEVNULL,
    )
    time.sleep(1)
    arrange_install_icons()
    subprocess.run(["sync"], check=False)
    detach_volume(mount)
    run(
        ["hdiutil", "convert", str(rw), "-format", "UDZO", "-o", str(dmg)],
        stdout=subprocess.DEVNULL,
    )
    rw.unlink(missing_ok=True)
    shutil.rmtree(stage, ignore_errors=True)
    return dmg


def detach_desk_volumes() -> None:
    volumes = Path("/Volumes")
    if not volumes.is_dir():
        return
    for path in sorted(volumes.iterdir()):
        if path.name.startswith("Desk Monitor"):
            subprocess.run(["hdiutil", "detach", str(path), "-force"], check=False)
            time.sleep(0.4)


def arrange_install_icons() -> None:
    script = """
tell application "Finder"
  tell disk "Desk Monitor"
    open
    set current view of container window to icon view
    set toolbar visible of container window to false
    set statusbar visible of container window to false
    set the bounds of container window to {220, 140, 780, 500}
    set viewOptions to the icon view options of container window
    set arrangement of viewOptions to not arranged
    set icon size of viewOptions to 128
    delay 0.4
    set position of item "Desk Monitor.app" of container window to {140, 180}
    try
      set position of item "Applications" of container window to {420, 180}
    end try
    close
    open
    update without registering applications
    delay 1
  end tell
end tell
"""
    result = subprocess.run(["osascript", "-e", script], text=True, capture_output=True)
    if result.returncode != 0:
        print(result.stderr.strip() or "Finder icon layout skipped.", file=sys.stderr)


def detach_volume(mount: Path) -> None:
    for _ in range(8):
        done = subprocess.run(["hdiutil", "detach", str(mount), "-quiet"])
        if done.returncode == 0:
            return
        time.sleep(1)
    run(["hdiutil", "detach", str(mount), "-force"])


def main() -> None:
    shutil.rmtree(DIST, ignore_errors=True)
    MACOS.mkdir(parents=True)
    FRAMEWORKS.mkdir(parents=True)
    RESOURCES.mkdir(parents=True)
    shutil.copy2(ROOT / "packaging" / "Info.plist", CONTENTS / "Info.plist")
    binary = MACOS / "DeskMonitor"
    shutil.copy2(binary_path(), binary)
    os.chmod(binary, 0o755)
    add_icon()
    copy_and_relink(binary)
    copy_gtk_data()
    run(["codesign", "--force", "--deep", "--sign", "-", str(APP)])
    dmg = add_dmg()
    print(APP)
    print(dmg)


if __name__ == "__main__":
    main()
