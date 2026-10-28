#!/usr/bin/env bash
# Builds a macOS .app bundle and a compressed .dmg for an iced (Rust) app.
set -euo pipefail

: "${APP_NAME:?}" "${APP_DISPLAY_NAME:?}" "${APP_IDENTIFIER:?}" "${APP_VERSION:?Could not read version from Cargo.toml}"

BIN_NAME="${BIN_NAME:-$APP_NAME}"
CARGO_ARGS="${CARGO_ARGS:-}"
ICON_PNG="${ICON_PNG:-}"
SIGN_IDENTITY="${SIGN_IDENTITY:--}"
MIN_MACOS="${MIN_MACOS:-11.0}"
UNIVERSAL="${UNIVERSAL:-0}"

MACOS_DIR="target/macos"
INSTALLER_DIR="target/installer"
APP_BUNDLE="$MACOS_DIR/$APP_NAME.app"
DMG_ROOT="$MACOS_DIR/dmg-root"

if [[ "$(uname -s)" != "Darwin" ]]; then
    echo "Error: DMG creation requires macOS." >&2
    exit 1
fi

# ---------------------------------------------------------------- build
echo "==> Building $APP_NAME v$APP_VERSION"
if [[ "$UNIVERSAL" == "1" ]]; then
    ARM=aarch64-apple-darwin
    X86=x86_64-apple-darwin
    for target in "$ARM" "$X86"; do
        rustup target add "$target"
        # shellcheck disable=SC2086
        cargo build --release $CARGO_ARGS --target "$target"
    done
    mkdir -p target/universal/release
    BINARY="target/universal/release/$BIN_NAME"
    lipo -create -output "$BINARY" \
        "target/$ARM/release/$BIN_NAME" \
        "target/$X86/release/$BIN_NAME"
    ARCH_LABEL="universal"
else
    # shellcheck disable=SC2086
    cargo build --release $CARGO_ARGS
    BINARY="target/release/$BIN_NAME"
    ARCH_LABEL="$(uname -m)"
fi

if [[ ! -x "$BINARY" ]]; then
    echo "Error: executable not found: $BINARY" >&2
    echo "Set BIN_NAME to the binary name Cargo produces (and CARGO_ARGS=\"-p <crate>\" if needed)." >&2
    exit 1
fi

# ---------------------------------------------------------------- bundle
echo "==> Creating app bundle"
rm -rf "$APP_BUNDLE" "$DMG_ROOT"
mkdir -p "$APP_BUNDLE/Contents/MacOS" "$APP_BUNDLE/Contents/Resources" "$DMG_ROOT" "$INSTALLER_DIR"

cp "$BINARY" "$APP_BUNDLE/Contents/MacOS/$APP_NAME"
chmod +x "$APP_BUNDLE/Contents/MacOS/$APP_NAME"

# ---------------------------------------------------------------- icon (optional)
ICON_KEY=""
if [[ -n "$ICON_PNG" && -f "$ICON_PNG" ]]; then
    echo "==> Generating AppIcon.icns from $ICON_PNG"
    ICONSET="$MACOS_DIR/AppIcon.iconset"
    rm -rf "$ICONSET"
    mkdir -p "$ICONSET"
    for size in 16 32 128 256 512; do
        double=$((size * 2))
        sips -z "$size" "$size" "$ICON_PNG" --out "$ICONSET/icon_${size}x${size}.png" >/dev/null
        sips -z "$double" "$double" "$ICON_PNG" --out "$ICONSET/icon_${size}x${size}@2x.png" >/dev/null
    done
    iconutil -c icns "$ICONSET" -o "$APP_BUNDLE/Contents/Resources/AppIcon.icns"
    rm -rf "$ICONSET"
    ICON_KEY="    <key>CFBundleIconFile</key>
    <string>AppIcon</string>"
else
    echo "==> No icon at '$ICON_PNG' (skipping). Put a 1024x1024 PNG there to get an app icon."
fi

# ---------------------------------------------------------------- Info.plist
cat > "$APP_BUNDLE/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key>
    <string>$APP_DISPLAY_NAME</string>
    <key>CFBundleDisplayName</key>
    <string>$APP_DISPLAY_NAME</string>
    <key>CFBundleExecutable</key>
    <string>$APP_NAME</string>
    <key>CFBundleIdentifier</key>
    <string>$APP_IDENTIFIER</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleShortVersionString</key>
    <string>$APP_VERSION</string>
    <key>CFBundleVersion</key>
    <string>$APP_VERSION</string>
    <key>LSMinimumSystemVersion</key>
    <string>$MIN_MACOS</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>NSPrincipalClass</key>
    <string>NSApplication</string>
$ICON_KEY
</dict>
</plist>
PLIST
plutil -lint "$APP_BUNDLE/Contents/Info.plist" >/dev/null

# ---------------------------------------------------------------- sign
echo "==> Signing (identity: $SIGN_IDENTITY)"
if [[ "$SIGN_IDENTITY" == "-" ]]; then
    codesign --force --deep --sign - "$APP_BUNDLE"
else
    codesign --force --deep --options runtime --timestamp --sign "$SIGN_IDENTITY" "$APP_BUNDLE"
fi
codesign --verify --deep --strict "$APP_BUNDLE"

# ---------------------------------------------------------------- dmg
echo "==> Creating DMG"
cp -R "$APP_BUNDLE" "$DMG_ROOT/"
ln -s /Applications "$DMG_ROOT/Applications"

DMG_FILE="$INSTALLER_DIR/$APP_NAME-$APP_VERSION-macos-$ARCH_LABEL.dmg"
rm -f "$DMG_FILE"

hdiutil create \
    -volname "$APP_DISPLAY_NAME" \
    -srcfolder "$DMG_ROOT" \
    -ov \
    -format UDZO \
    "$DMG_FILE" >/dev/null

if [[ "$SIGN_IDENTITY" != "-" ]]; then
    codesign --force --sign "$SIGN_IDENTITY" --timestamp "$DMG_FILE"
fi

rm -rf "$DMG_ROOT"

echo
echo "DMG created successfully:"
ls -lh "$DMG_FILE"