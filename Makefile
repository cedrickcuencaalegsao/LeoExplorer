.PHONY: all run build release clean check

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