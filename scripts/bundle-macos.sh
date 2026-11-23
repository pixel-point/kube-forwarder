#!/usr/bin/env bash
# Builds "Kube Forwarder.app" and kube-forwarder.dmg into dist/.
#
#   scripts/bundle-macos.sh               # native build (arm64 on Apple Silicon)
#   scripts/bundle-macos.sh --universal   # arm64 + x86_64
#
# Set MACOS_SIGNING_IDENTITY to sign with a Developer ID certificate, otherwise the app
# is signed ad hoc (fine for local use; distribution requires signing and notarization).
set -euo pipefail

cd "$(dirname "$0")/.."
ROOT="$(pwd)"
NAME="Kube Forwarder"
BIN="kube-forwarder"
VERSION="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)"
DIST="$ROOT/dist"
APP="$DIST/$NAME.app"
export MACOSX_DEPLOYMENT_TARGET="${MACOSX_DEPLOYMENT_TARGET:-11.0}"

UNIVERSAL=false
[[ "${1:-}" == "--universal" ]] && UNIVERSAL=true

if $UNIVERSAL; then
  rustup target add aarch64-apple-darwin x86_64-apple-darwin >/dev/null
  cargo build --release -p "$BIN" --target aarch64-apple-darwin
  cargo build --release -p "$BIN" --target x86_64-apple-darwin
  EXECUTABLE="$DIST/$BIN"
  mkdir -p "$DIST"
  lipo -create \
    "target/aarch64-apple-darwin/release/$BIN" \
    "target/x86_64-apple-darwin/release/$BIN" \
    -output "$EXECUTABLE"
else
  cargo build --release -p "$BIN"
  EXECUTABLE="target/release/$BIN"
fi

rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp "$EXECUTABLE" "$APP/Contents/MacOS/$BIN"

# Icon
ICONSET="$(mktemp -d)/AppIcon.iconset"
mkdir -p "$ICONSET"
SOURCE_ICON="crates/kube-forwarder/assets/app-icon.png"
for size in 16 32 128 256 512; do
  sips -z $size $size "$SOURCE_ICON" --out "$ICONSET/icon_${size}x${size}.png" >/dev/null
  sips -z $((size * 2)) $((size * 2)) "$SOURCE_ICON" --out "$ICONSET/icon_${size}x${size}@2x.png" >/dev/null
done
iconutil -c icns "$ICONSET" -o "$APP/Contents/Resources/AppIcon.icns"

cat > "$APP/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key>
  <string>$NAME</string>
  <key>CFBundleDisplayName</key>
  <string>$NAME</string>
  <key>CFBundleIdentifier</key>
  <string>com.pixelpoint.kube-forwarder</string>
  <key>CFBundleVersion</key>
  <string>$VERSION</string>
  <key>CFBundleShortVersionString</key>
  <string>$VERSION</string>
  <key>CFBundleExecutable</key>
  <string>$BIN</string>
  <key>CFBundleIconFile</key>
  <string>AppIcon</string>
  <key>CFBundlePackageType</key>
  <string>APPL</string>
  <key>LSMinimumSystemVersion</key>
  <string>$MACOSX_DEPLOYMENT_TARGET</string>
  <key>LSApplicationCategoryType</key>
  <string>public.app-category.developer-tools</string>
  <key>NSHighResolutionCapable</key>
  <true/>
  <key>NSHumanReadableCopyright</key>
  <string>MIT License</string>
</dict>
</plist>
PLIST

if [[ -n "${MACOS_SIGNING_IDENTITY:-}" ]]; then
  codesign --force --options runtime --timestamp --sign "$MACOS_SIGNING_IDENTITY" "$APP"
else
  codesign --force --sign - "$APP"
fi
codesign --verify --strict "$APP"

# Disk image with a link to /Applications
STAGING="$(mktemp -d)"
cp -R "$APP" "$STAGING/"
ln -s /Applications "$STAGING/Applications"
rm -f "$DIST/$BIN.dmg"
hdiutil create -quiet -volname "$NAME" -srcfolder "$STAGING" -ov -format UDZO "$DIST/$BIN.dmg"
rm -rf "$STAGING"

echo "Built $APP"
echo "Built $DIST/$BIN.dmg"
lipo -info "$APP/Contents/MacOS/$BIN"
