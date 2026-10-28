.DEFAULT_GOAL := all
SHELL := /bin/bash
.SHELLFLAGS := -eu -o pipefail -c

.PHONY: all run build release check clean dmg dmg-universal install-mac clean-installer add-icon

# ---------------------------------------------------------------------------
# Project configuration (override on the CLI, e.g. `make dmg BIN_NAME=leoexplorer`)
# ---------------------------------------------------------------------------
APP_NAME         ?= LeoExplorer
APP_DISPLAY_NAME ?= Leo Explorer
APP_IDENTIFIER   ?= com.cedrickalegsao.leoexplorer

# Name of the executable Cargo produces in target/release/
# (the [[bin]] name, or the package name if there is no [[bin]]).
BIN_NAME         ?= $(APP_NAME)

# Extra cargo flags, e.g. CARGO_ARGS="-p leoexplorer-gui"
CARGO_ARGS       ?=

# 1024x1024 PNG used to generate the .icns (optional)
ICON_PNG         ?= assets/AppIcon.png

# "-" = ad-hoc signing. For distribution use e.g.
# SIGN_IDENTITY="Developer ID Application: Your Name (TEAMID)"
SIGN_IDENTITY    ?= -
MIN_MACOS        ?= 11.0
UNIVERSAL        ?= 0

# Paths
ICON_DIR := assets/icons
ICON_RS  := leoexplorer-gui/src/icons.rs

# Version from the root Cargo.toml
APP_VERSION := $(shell sed -n 's/^[[:space:]]*version[[:space:]]*=[[:space:]]*"\([^"]*\)".*/\1/p' Cargo.toml | head -n 1)

# Lucide icon source
LUCIDE_URL := https://raw.githubusercontent.com/lucide-icons/lucide/main/icons

export APP_NAME APP_DISPLAY_NAME APP_IDENTIFIER BIN_NAME CARGO_ARGS ICON_PNG \
	   SIGN_IDENTITY MIN_MACOS UNIVERSAL APP_VERSION

# ---------------------------------------------------------------------------
# Cargo shortcuts
# ---------------------------------------------------------------------------
all: build

run:
	cargo run

build:
	cargo build

release:
	cargo build --release

check:
	cargo check

clean:
	cargo clean

# ---------------------------------------------------------------------------
# macOS installer
# ---------------------------------------------------------------------------

# .app bundle + compressed DMG for the current architecture
dmg:
	@bash scripts/build-dmg.sh

# Universal (Apple Silicon + Intel) DMG
dmg-universal:
	@$(MAKE) --no-print-directory dmg UNIVERSAL=1

install-mac: dmg

clean-installer:
	rm -rf target/installer target/macos target/universal

# ---------------------------------------------------------------------------
# Add one or more Lucide icons
#   make add-icon sun
#   make add-icon sun moon hard-drive
#   make add-icon ICONS=check
# ---------------------------------------------------------------------------
ICONS ?= $(filter-out add-icon,$(MAKECMDGOALS))

add-icon:
	@if [ -z "$(strip $(ICONS))" ]; then \
	    echo "Usage: make add-icon <lucide-icon-name> [more names...]"; \
	    echo "Example: make add-icon sun moon hard-drive"; \
	    exit 1; \
	fi
	@mkdir -p "$(ICON_DIR)"
	@touch "$(ICON_RS)"
	@for name in $(ICONS); do \
	    file="$(ICON_DIR)/$$name.svg"; \
	    url="$(LUCIDE_URL)/$$name.svg"; \
	    if curl -fsSL "$$url" -o "$$file"; then \
	        sed -i.bak 's/stroke-width="2"/stroke-width="1.5"/g' "$$file" && rm -f "$$file.bak"; \
	        fn=$$(echo "$$name" | tr '-' '_'); \
	        if grep -Eq "^[[:space:]]*icon_fn!\($$fn," "$(ICON_RS)"; then \
	            echo "Icon $$name already registered in $(ICON_RS)"; \
	        else \
	            printf 'icon_fn!(%s, "%s.svg");\n' "$$fn" "$$name" >> "$(ICON_RS)"; \
	            echo "Registered icon: $$fn"; \
	        fi; \
	    else \
	        rm -f "$$file"; \
	        echo "Error: Lucide icon '$$name' not found."; \
	    fi; \
	done

# Prevent icon names from being interpreted as Make targets
ifneq (,$(filter add-icon,$(MAKECMDGOALS)))
$(ICONS):
	@:
endif