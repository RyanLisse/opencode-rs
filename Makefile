# Makefile for the OpenCode-RS Project

# Use .PHONY to ensure these targets run even if files with the same name exist.
.PHONY: all build check test lint clean help

# Default target runs when you just type 'make'
all: build

## --------------------------------------
## Development Commands
## --------------------------------------

# Build all crates in the workspace in debug mode
build:
	@echo "Building workspace..."
	@cargo build --workspace

# Check all crates for errors without building executables (faster)
check:
	@echo "Checking workspace..."
	@cargo check --workspace

# Run all tests in the workspace
test:
	@echo "Running tests..."
	@cargo test --workspace -- --nocapture

# Run the linter (clippy) and fail on any warnings
lint:
	@echo "Linting workspace..."
	@cargo clippy --workspace -- -D warnings

## --------------------------------------
## Housekeeping
## --------------------------------------

# Clean up build artifacts
clean:
	@echo "Cleaning workspace..."
	@cargo clean

# Self-documenting help command
help:
	@echo "Available commands:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'