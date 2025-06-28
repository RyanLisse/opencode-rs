# Makefile for the OpenCode-RS Project

# Use .PHONY to ensure these targets run even if files with the same name exist.
.PHONY: all build check test lint clean help install release format docs coverage benchmark gui-dev gui-build gui-test integration-test security-audit cross-platform-test full-test package

# Default target runs when you just type 'make'
all: build

## --------------------------------------
## Development Commands
## --------------------------------------

# Build all crates in the workspace in debug mode
build: ## Build all crates in debug mode
	@echo "Building workspace..."
	@cargo build --workspace

# Build in release mode with optimizations
build-release: ## Build all crates in release mode
	@echo "Building workspace in release mode..."
	@cargo build --workspace --release

# Check all crates for errors without building executables (faster)
check: ## Check all crates for errors without building
	@echo "Checking workspace..."
	@cargo check --workspace

# Run all tests in the workspace
test: ## Run all unit tests
	@echo "Running tests..."
	@cargo test --workspace -- --nocapture

# Run the linter (clippy) and fail on any warnings
lint: ## Run clippy linter
	@echo "Linting workspace..."
	@cargo clippy --workspace -- -D warnings

# Format all code
format: ## Format all Rust code
	@echo "Formatting code..."
	@cargo fmt --all

# Check if code is properly formatted
format-check: ## Check if code is properly formatted
	@echo "Checking code formatting..."
	@cargo fmt --all -- --check

## --------------------------------------
## Testing & Quality Assurance
## --------------------------------------

# Run comprehensive test suite
full-test: ## Run the complete test suite
	@echo "Running full test suite..."
	@./scripts/test-full-suite.sh

# Run integration tests
integration-test: ## Run integration tests
	@echo "Running integration tests..."
	@cargo test --test '*' --verbose

# Run security audit
security-audit: ## Run security audit with cargo-audit
	@echo "Running security audit..."
	@cargo audit || (echo "Installing cargo-audit..." && cargo install cargo-audit && cargo audit)

# Generate code coverage report
coverage: ## Generate code coverage report
	@echo "Generating code coverage..."
	@cargo tarpaulin --all-features --workspace --timeout 120 --out html --output-dir coverage || \
	(echo "Installing cargo-tarpaulin..." && cargo install cargo-tarpaulin && \
	 cargo tarpaulin --all-features --workspace --timeout 120 --out html --output-dir coverage)

# Performance benchmarking
benchmark: ## Run performance benchmarks
	@echo "Running performance benchmarks..."
	@./scripts/benchmark.sh

# Cross-platform compatibility tests
cross-platform-test: ## Test cross-platform compatibility
	@echo "Testing cross-platform compatibility..."
	@cargo test test_cross_platform_compatibility --release

## --------------------------------------
## GUI Development
## --------------------------------------

# Install GUI dependencies
gui-deps: ## Install GUI dependencies
	@echo "Installing GUI dependencies..."
	@cd crates/opencode-gui && pnpm install

# Run GUI in development mode
gui-dev: gui-deps ## Start GUI development server
	@echo "Starting GUI development server..."
	@cd crates/opencode-gui && pnpm dev

# Build GUI for production
gui-build: gui-deps ## Build GUI for production
	@echo "Building GUI for production..."
	@cd crates/opencode-gui && pnpm build

# Run GUI tests
gui-test: gui-deps ## Run GUI tests
	@echo "Running GUI tests..."
	@cd crates/opencode-gui && pnpm test:ci

# Run GUI linting
gui-lint: gui-deps ## Run GUI linting
	@echo "Running GUI linting..."
	@cd crates/opencode-gui && pnpm lint

# Type check GUI
gui-type-check: gui-deps ## Type check GUI code
	@echo "Type checking GUI..."
	@cd crates/opencode-gui && pnpm type-check

## --------------------------------------
## Documentation
## --------------------------------------

# Generate documentation
docs: ## Generate Rust documentation
	@echo "Generating documentation..."
	@cargo doc --no-deps --all-features --open

# Check documentation for issues
docs-check: ## Check documentation for issues
	@echo "Checking documentation..."
	@cargo doc --no-deps --all-features 2>&1 | grep -q "warning:" && exit 1 || exit 0

## --------------------------------------
## Release & Packaging
## --------------------------------------

# Create a release build with all optimizations
release: format lint test build-release docs ## Create a complete release build
	@echo "Release build completed successfully!"

# Package application for distribution
package: release ## Package application for distribution
	@echo "Packaging application..."
	@mkdir -p dist
	@cp target/release/opencode dist/
	@tar -czf dist/opencode-$(shell date +%Y%m%d-%H%M%S).tar.gz -C dist opencode
	@echo "Package created in dist/"

# Install locally (for development)
install: build-release ## Install locally for development
	@echo "Installing opencode locally..."
	@cargo install --path crates/cli --force

# Create installers using the install script
create-installers: ## Create cross-platform installers
	@echo "Creating installers..."
	@chmod +x scripts/install.sh
	@echo "Install script ready at scripts/install.sh"

## --------------------------------------
## Container & Deployment
## --------------------------------------

# Build Docker image (if Dockerfile exists)
docker-build: ## Build Docker image
	@echo "Building Docker image..."
	@docker build -t opencode-rs:latest . || echo "Dockerfile not found, skipping Docker build"

# Run application in container
docker-run: docker-build ## Run application in container
	@echo "Running in container..."
	@docker run --rm -it opencode-rs:latest || echo "Docker not available"

## --------------------------------------
## Development Tools
## --------------------------------------

# Install development tools
dev-tools: ## Install development tools
	@echo "Installing development tools..."
	@cargo install cargo-audit cargo-tarpaulin cargo-watch || true
	@echo "Development tools installation completed"

# Watch for changes and rebuild
watch: ## Watch for changes and rebuild
	@echo "Watching for changes..."
	@cargo watch -x "build --workspace" || (echo "Installing cargo-watch..." && cargo install cargo-watch && cargo watch -x "build --workspace")

# Clean all build artifacts and dependencies
deep-clean: clean ## Deep clean including dependencies
	@echo "Deep cleaning..."
	@cargo clean
	@rm -rf target/
	@rm -rf crates/opencode-gui/node_modules/
	@rm -rf crates/opencode-gui/dist/
	@rm -rf coverage/
	@rm -rf benchmarks/results/

## --------------------------------------
## CI/CD Simulation
## --------------------------------------

# Simulate CI pipeline locally
ci: format-check lint test security-audit docs-check ## Simulate CI pipeline locally
	@echo "✅ CI simulation completed successfully!"

# Pre-commit hook simulation
pre-commit: format lint test ## Run pre-commit checks
	@echo "✅ Pre-commit checks passed!"

# Pre-release checks
pre-release: full-test security-audit docs benchmark ## Run pre-release checks
	@echo "✅ Pre-release checks completed!"

## --------------------------------------
## Housekeeping
## --------------------------------------

# Clean up build artifacts
clean: ## Clean build artifacts
	@echo "Cleaning workspace..."
	@cargo clean

# Display project status
status: ## Display project status
	@echo "=== OpenCode-RS Project Status ==="
	@echo "Rust version: $(shell rustc --version)"
	@echo "Cargo version: $(shell cargo --version)"
	@echo "Project root: $(shell pwd)"
	@echo "Git branch: $(shell git branch --show-current 2>/dev/null || echo 'Not a git repository')"
	@echo "Git status: $(shell git status --porcelain 2>/dev/null | wc -l | xargs echo) files changed"
	@echo "Last commit: $(shell git log -1 --pretty=format:'%h - %s (%cr)' 2>/dev/null || echo 'No commits found')"
	@echo ""
	@echo "Workspace crates:"
	@cargo metadata --format-version 1 | jq -r '.workspace_members[]' | sed 's/^/  - /'

# Self-documenting help command
help: ## Show this help message
	@echo "OpenCode-RS Development Commands"
	@echo "================================"
	@echo ""
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'
	@echo ""
	@echo "Quick Start:"
	@echo "  make build          # Build the project"
	@echo "  make test           # Run tests"
	@echo "  make ci             # Run full CI simulation"
	@echo "  make release        # Create release build"
	@echo "  make gui-dev        # Start GUI development"
	@echo ""
	@echo "For more information, see README.md"