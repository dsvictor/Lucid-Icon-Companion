#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

TARGET="${1:-aarch64-apple-darwin}"
APP="dist/Lucide Companion.app"
BIN="target/$TARGET/release/lucide-icon-companion"
DMG_ROOT="dist/dmg-root"
DMG="dist/Lucide-Companion-macOS-arm64.dmg"

BUILD_COMMIT="$(git rev-parse --short HEAD)"
BUILD_BRANCH="$(git branch --show-current)"
BUILD_DATE="$(date '+%Y-%m-%d %H:%M:%S %Z')"
LUCIDE_BUILD_CHANNEL="preview" \
LUCIDE_BUILD_BRANCH="$BUILD_BRANCH" \
LUCIDE_BUILD_COMMIT="$BUILD_COMMIT" \
LUCIDE_BUILD_DATE="$BUILD_DATE" \
cargo build --release --target "$TARGET"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
install -m 755 "$BIN" "$APP/Contents/MacOS/LucideCompanion"

# Render the supplied SVG mark into a native macOS .icns file.
ICONSET="dist/AppIcon.iconset"
mkdir -p "$ICONSET"
rsvg-convert --width 1024 --height 1024 assets/app-icon.svg > dist/AppIcon-1024.png
for SIZE in 16 32 128 256 512; do
  sips -z "$SIZE" "$SIZE" dist/AppIcon-1024.png --out "$ICONSET/icon_${SIZE}x${SIZE}.png" >/dev/null
done
sips -z 32 32 dist/AppIcon-1024.png --out "$ICONSET/icon_16x16@2x.png" >/dev/null
sips -z 64 64 dist/AppIcon-1024.png --out "$ICONSET/icon_32x32@2x.png" >/dev/null
sips -z 256 256 dist/AppIcon-1024.png --out "$ICONSET/icon_128x128@2x.png" >/dev/null
sips -z 512 512 dist/AppIcon-1024.png --out "$ICONSET/icon_256x256@2x.png" >/dev/null
sips -z 1024 1024 dist/AppIcon-1024.png --out "$ICONSET/icon_512x512@2x.png" >/dev/null
iconutil -c icns "$ICONSET" -o "$APP/Contents/Resources/AppIcon.icns"
rm -rf "$ICONSET" dist/AppIcon-1024.png

cat > "$APP/Contents/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleName</key><string>Lucide Companion</string>
  <key>CFBundleDisplayName</key><string>Lucide Companion for FigCraft</string>
  <key>CFBundleIdentifier</key><string>dev.jayanta.lucide-icon-companion.preview</string>
  <key>CFBundleVersion</key><string>0.2.0</string>
  <key>CFBundleShortVersionString</key><string>0.2.0</string>
  <key>LucideBuildChannel</key><string>preview</string>
  <key>LucideBuildBranch</key><string>${BUILD_BRANCH}</string>
  <key>LucideBuildCommit</key><string>${BUILD_COMMIT}</string>
  <key>LucideBuildDate</key><string>${BUILD_DATE}</string>
  <key>CFBundleExecutable</key><string>LucideCompanion</string>
  <key>CFBundleIconFile</key><string>AppIcon</string>
  <key>LSMinimumSystemVersion</key><string>13.0</string>
  <key>NSHighResolutionCapable</key><true/>
</dict>
</plist>
PLIST

rm -rf "$DMG_ROOT"
mkdir -p "$DMG_ROOT"
cp -R "$APP" "$DMG_ROOT/"
ln -s /Applications "$DMG_ROOT/Applications"
hdiutil create -volname "Lucide Companion" -srcfolder "$DMG_ROOT" -ov -format UDZO "$DMG"
rm -rf "$DMG_ROOT"
