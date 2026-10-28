.DEFAULT_GOAL := all
.ONESHELL:
SHELL := /bin/bash
.SHELLFLAGS := -eu -o pipefail -c

.PHONY: all run build release check clean dmg install-mac clean-installer add-icon

# Project configuration
APP_NAME := LeoExplorer
APP_DISPLAY_NAME := Leo Explorer
APP_IDENTIFIER := com.cedrickalegsao.leoexplorer

# Paths
ICON_DIR := assets/icons
ICON_RS := leoexplorer-gui/src/icons.rs
INSTALLER_DIR := target/installer
MACOS_DIR := target/macos
APP_BUNDLE := $(MACOS_DIR)/$(APP_NAME).app
DMG_ROOT := $(MACOS_DIR)/dmg-root
BINARY := target/release/$(APP_NAME)

# Version from the root Cargo.toml
APP_VERSION := $(shell sed -n 's/^[[:space:]]*version[[:space:]]*=[[:space:]]*"\([^"]*\)".*/\1/p' Cargo.toml | head -n 1)

# Lucide icon source
LUCIDE_URL := https://raw.githubusercontent.com/lucide-icons/lucide/main/icons

# Default command
all: build

# Run the application
run:
	cargo run

# Build the application in debug mode
build:
	cargo build

# Build the application in release mode
release:
	cargo build --release

# Check the code without producing an executable
check:
	cargo check

# Remove Cargo build artifacts
clean:
	cargo clean

# Build a macOS .app bundle and compressed DMG installer
dmg:
	@if [[ "$$(uname -s)" != "Darwin" ]]; then
		echo "Error: DMG creation requires macOS."
		exit 1
	fi

	@if [[ -z "$(APP_VERSION)" ]]; then
		echo "Error: Could not read version from Cargo.toml."
		exit 1
	fi

	echo "Building $(APP_NAME) v$(APP_VERSION)..."
	cargo build --release

	if [[ ! -x "$(BINARY)" ]]; then
		echo "Error: Executable not found: $(BINARY)"
		echo "Check the binary target name in Cargo.toml."
		exit 1
	fi

	echo "Creating application bundle..."
	rm -rf "$(APP_BUNDLE)" "$(DMG_ROOT)"
	mkdir -p \
		"$(APP_BUNDLE)/Contents/MacOS" \
		"$(APP_BUNDLE)/Contents/Resources" \
		"$(DMG_ROOT)" \
		"$(INSTALLER_DIR)"

	cp "$(BINARY)" "$(APP_BUNDLE)/Contents/MacOS/$(APP_NAME)"
	chmod +x "$(APP_BUNDLE)/Contents/MacOS/$(APP_NAME)"

	cat > "$(APP_BUNDLE)/Contents/Info.plist" <<'PLIST'
	<?xml version="1.0" encoding="UTF-8"?>
	<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
	<plist version="1.0">
	<dict>
	    <key>CFBundleName</key>
	    <string>Leo Explorer</string>
	    <key>CFBundleDisplayName</key>
	    <string>Leo Explorer</string>
	    <key>CFBundleExecutable</key>
	    <string>LeoExplorer</string>
	    <key>CFBundleIdentifier</key>
	    <string>com.cedrickalegsao.leoexplorer</string>
	    <key>CFBundlePackageType</key>
	    <string>APPL</string>
	    <key>CFBundleInfoDictionaryVersion</key>
	    <string>6.0</string>
	    <key>CFBundleShortVersionString</key>
	    <string>$(APP_VERSION)</string>
	    <key>CFBundleVersion</key>
	    <string>$(APP_VERSION)</string>
	    <key>LSMinimumSystemVersion</key>
	    <string>13.0</string>
	</dict>
	</plist>
	PLIST

	/usr/libexec/PlistBuddy -c "Print :CFBundleIdentifier" "$(APP_BUNDLE)/Contents/Info.plist" >/dev/null

	echo "Preparing DMG contents..."
	cp -R "$(APP_BUNDLE)" "$(DMG_ROOT)/"
	ln -s /Applications "$(DMG_ROOT)/Applications"

	DMG_FILE="$(INSTALLER_DIR)/$(APP_NAME)-$(APP_VERSION)-macos-$$(uname -m).dmg"
	rm -f "$$DMG_FILE"

	echo "Creating DMG..."
	hdiutil create \
		-volname "$(APP_DISPLAY_NAME)" \
		-srcfolder "$(DMG_ROOT)" \
		-ov \
		-format UDZO \
		"$$DMG_FILE"

	echo ""
	echo "DMG created successfully:"
	ls -lh "$$DMG_FILE"

# Alias for creating the macOS installer
install-mac: dmg

# Remove generated installers and app bundles
clean-installer:
	rm -rf "$(INSTALLER_DIR)" "$(MACOS_DIR)"

# Add one or more Lucide icons
# Examples:
#   make add-icon sun
#   make add-icon sun moon hard-drive
#   make add-icon ICONS=check

ICONS ?= $(filter-out add-icon,$(MAKECMDGOALS))

add-icon:
	if [[ -z "$(strip $(ICONS))" ]]; then
		echo "Usage: make add-icon <lucide-icon-name> [more names...]"
		echo "Example: make add-icon sun moon hard-drive"
		exit 1
	fi

	mkdir -p "$(ICON_DIR)"
	touch "$(ICON_RS)"

	for name in $(ICONS); do
		file="$(ICON_DIR)/$$name.svg"
		url="$(LUCIDE_URL)/$$name.svg"

		if curl -fsSL "$$url" -o "$$file"; then
			sed -i '' 's/stroke-width="2"/stroke-width="1.5"/g' "$$file"

			fn=$$(echo "$$name" | tr '-' '_')

			if grep -Eq "^[[:space:]]*icon_fn!\($$fn," "$(ICON_RS)"; then
				echo "Icon $$name already registered in $(ICON_RS)"
			else
				printf 'icon_fn!(%s, "%s.svg");\n' "$$fn" "$$name" >> "$(ICON_RS)"
				echo "Registered icon: $$fn"
			fi
		else
			rm -f "$$file"
			echo "Error: Lucide icon '$$name' not found."
		fi
	done

# Prevent icon names from being interpreted as Make targets
ifneq (,$(filter add-icon,$(MAKECMDGOALS)))
ICONS := $(filter-out add-icon,$(MAKECMDGOALS))
$(ICONS):
	@:
endif