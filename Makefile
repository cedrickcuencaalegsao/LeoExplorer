.PHONY: all run build release clean check add-icon

# Paths (adjust if your layout changes)
ICON_DIR  := assets/icons
ICON_RS   := leoexplorer-gui/src/icons.rs
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

# Remove generated build artifacts
clean:
	cargo clean

# Add one or more Lucide icons:
#   make add-icon sun
#   make add-icon sun moon hard-drive
#   make add-icon ICONS=check        (use this form if the icon is named like a make target)
ICONS ?= $(filter-out add-icon,$(MAKECMDGOALS))

add-icon:
	@if [ -z "$(ICONS)" ]; then \
		echo "Usage: make add-icon <lucide-icon-name> [more names...]"; \
		exit 1; \
	fi
	@mkdir -p $(ICON_DIR)
	@for name in $(ICONS); do \
		file="$(ICON_DIR)/$$name.svg"; \
		if curl -fsSL -o "$$file" "$(LUCIDE_URL)/$$name.svg"; then \
			sed -i.bak 's/stroke-width="2"/stroke-width="1.5"/' "$$file" && rm -f "$$file.bak"; \
			fn=$$(echo "$$name" | tr '-' '_'); \
			if grep -q "icon_fn!($$fn," $(ICON_RS); then \
				echo "Updated $$file ($$fn already registered in $(ICON_RS))"; \
			else \
				echo "icon_fn!($$fn, \"$$name.svg\");" >> $(ICON_RS); \
				echo "Added $$file -> icon::$$fn(size)"; \
			fi; \
		else \
			rm -f "$$file"; \
			echo "Icon '$$name' not found on Lucide (browse https://lucide.dev/icons)"; \
		fi; \
	done

# Swallow the icon names so make doesn't treat them as targets
ifneq (,$(filter add-icon,$(MAKECMDGOALS)))
$(filter-out add-icon,$(MAKECMDGOALS)):
	@:
endif