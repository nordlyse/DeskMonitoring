# Requires GTK4 from gvsbuild at C:\gtk-build\gtk\x64\release
$ErrorActionPreference = "Stop"

$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
Set-Location $Root

$Gtk = "C:\gtk-build\gtk\x64\release"
if (-not (Test-Path $Gtk)) {
    throw "GTK4 prefix not found at $Gtk. Install with: pipx install gvsbuild; gvsbuild build gtk4"
}

$env:PKG_CONFIG_PATH = "$Gtk\lib\pkgconfig"
$env:Path = "$Gtk\bin;" + $env:Path
if ($env:LIB) {
    $env:LIB = "$Gtk\lib;" + $env:LIB
} else {
    $env:LIB = "$Gtk\lib"
}

python packaging/icon.py
cargo build --release

$BinDir = Join-Path $Root "target\release"
if ($env:CARGO_TARGET_DIR) {
    $Alt = Join-Path $env:CARGO_TARGET_DIR "release\desk-monitoring.exe"
    if (Test-Path $Alt) { $Exe = $Alt } else { $Exe = Join-Path $BinDir "desk-monitoring.exe" }
} else {
    $Exe = Join-Path $BinDir "desk-monitoring.exe"
}

$Out = Join-Path $Root "dist\DeskMonitor-windows"
if (Test-Path $Out) { Remove-Item -Recurse -Force $Out }
New-Item -ItemType Directory -Path $Out | Out-Null
Copy-Item $Exe (Join-Path $Out "desk-monitoring.exe")
Copy-Item "$Gtk\bin\*.dll" $Out
if (Test-Path "$Gtk\lib\gdk-pixbuf-2.0") {
    Copy-Item "$Gtk\lib\gdk-pixbuf-2.0" (Join-Path $Out "lib\gdk-pixbuf-2.0") -Recurse
}
if (Test-Path "$Gtk\share") {
    Copy-Item "$Gtk\share" (Join-Path $Out "share") -Recurse
}

$Launcher = Join-Path $Out "DeskMonitor.bat"
@"
@echo off
setlocal
set BASE=%~dp0
set PATH=%BASE%;%BASE%bin;%PATH%
set GDK_PIXBUF_MODULEDIR=%BASE%lib\gdk-pixbuf-2.0\2.10.0\loaders
set GDK_PIXBUF_MODULE_FILE=%BASE%lib\gdk-pixbuf-2.0\2.10.0\loaders.cache
set GSETTINGS_SCHEMA_DIR=%BASE%share\glib-2.0\schemas
set XDG_DATA_DIRS=%BASE%share
start "" "%BASE%desk-monitoring.exe"
"@ | Set-Content -Path $Launcher -Encoding ASCII

$Zip = Join-Path $Root "dist\DeskMonitor-windows.zip"
if (Test-Path $Zip) { Remove-Item $Zip }
Compress-Archive -Path (Join-Path $Out "*") -DestinationPath $Zip
Write-Output $Zip
