#!/usr/bin/env bash
set -euo pipefail

TARGET="${1:-aarch64-apple-darwin}"
APP="dist/Lucide Companion.app"
BIN="target/$TARGET/release/lucid-icon-companion"

cargo build --release --target "$TARGET"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
install -m 755 "$BIN" "$APP/Contents/MacOS/LucideCompanion"

cat > "$APP/Contents/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key><string>Lucide Companion</string>
  <key>CFBundleDisplayName</key><string>Lucide Companion for FigCraft</string>
  <key>CFBundleIdentifier</key><string>dev.jayanta.lucide-icon-companion</string>
  <key>CFBundleVersion</key><string>0.1.0</string>
  <key>CFBundleShortVersionString</key><string>0.1.0</string>
  <key>CFBundleExecutable</key><string>LucideCompanion</string>
  <key>LSMinimumSystemVersion</key><string>13.0</string>
  <key>NSHighResolutionCapable</key><true/>
</dict>
</plist>
PLIST

hdiutil create -volname "Lucide Companion" -srcfolder "$APP" -ov -format UDZO "dist/Lucide-Companion-macOS-arm64.dmg"
