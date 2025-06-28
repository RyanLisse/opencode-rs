#!/bin/bash
set -e

# Comprehensive test suite for OpenCode-RS
# This script runs all tests across the entire project

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Print colored output
print_info() {
    printf "${GREEN}[INFO]${NC} %s\n" "$1"
}

print_warn() {
    printf "${YELLOW}[WARN]${NC} %s\n" "$1"
}

print_error() {
    printf "${RED}[ERROR]${NC} %s\n" "$1" >&2
}

print_section() {
    printf "\n${BLUE}=== %s ===${NC}\n" "$1"
}

# Test result tracking
TESTS_PASSED=0
TESTS_FAILED=0
FAILED_TESTS=()

# Function to run a test and track results
run_test() {
    local test_name="$1"
    local test_command="$2"
    
    print_info "Running: $test_name"
    
    if eval "$test_command"; then
        print_info "✅ PASSED: $test_name"
        ((TESTS_PASSED++))
    else
        print_error "❌ FAILED: $test_name"
        ((TESTS_FAILED++))
        FAILED_TESTS+=("$test_name")
    fi
}

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

print_section "OpenCode-RS Full Test Suite"

print_info "Starting comprehensive test suite..."
print_info "Project root: $(pwd)"

# Check prerequisites
print_section "Checking Prerequisites"

if ! command_exists cargo; then
    print_error "Rust/Cargo not found. Please install Rust."
    exit 1
fi

if ! command_exists node; then
    print_warn "Node.js not found. GUI tests will be skipped."
fi

if ! command_exists pnpm; then
    print_warn "pnpm not found. GUI tests will be skipped."
fi

if ! command_exists docker; then
    print_warn "Docker not found. Container tests will be skipped."
fi

print_info "Prerequisites check completed"

# Rust tests
print_section "Rust Core Tests"

run_test "Cargo format check" "cargo fmt --all -- --check"
run_test "Cargo clippy" "cargo clippy --all-targets --all-features -- -D warnings"
run_test "Core library unit tests" "cargo test --lib --verbose"
run_test "Core library doc tests" "cargo test --doc"
run_test "Integration tests" "cargo test --test '*' --verbose"

# CLI tests
print_section "CLI Tests"

run_test "CLI build" "cargo build --bin opencode --release"
run_test "CLI help command" "./target/release/opencode --help"
run_test "CLI version command" "./target/release/opencode --version"

# GUI tests (if available)
if command_exists node && command_exists pnpm && [ -d "crates/opencode-gui" ]; then
    print_section "GUI Frontend Tests"
    
    cd crates/opencode-gui
    
    run_test "Frontend dependencies install" "pnpm install --frozen-lockfile"
    run_test "Frontend TypeScript check" "pnpm type-check"
    run_test "Frontend linting" "pnpm lint"
    run_test "Frontend unit tests" "pnpm test:ci"
    run_test "Frontend build" "pnpm build"
    
    cd ../..
else
    print_warn "Skipping GUI tests - Node.js/pnpm not available or GUI directory not found"
fi

# Performance tests
print_section "Performance Tests"

run_test "Performance benchmarks" "cargo test test_performance_benchmarks --release"
run_test "Memory usage tests" "cargo test test_memory_usage --release"

# Security tests
print_section "Security Tests"

if command_exists cargo-audit; then
    run_test "Security audit" "cargo audit"
else
    print_warn "cargo-audit not found. Installing..."
    if cargo install cargo-audit; then
        run_test "Security audit" "cargo audit"
    else
        print_error "Failed to install cargo-audit"
        ((TESTS_FAILED++))
        FAILED_TESTS+=("Security audit")
    fi
fi

# Cross-platform compatibility tests
print_section "Cross-Platform Tests"

run_test "Cross-platform compatibility" "cargo test test_cross_platform_compatibility"

# Documentation tests
print_section "Documentation Tests"

run_test "Documentation build" "cargo doc --no-deps --all-features"
run_test "Documentation links check" "cargo doc --no-deps --all-features 2>&1 | grep -q 'warning:' && exit 1 || exit 0"

# Coverage test (if tarpaulin is available)
print_section "Code Coverage"

if command_exists cargo-tarpaulin; then
    run_test "Code coverage" "cargo tarpaulin --all-features --workspace --timeout 120 --out json --output-dir coverage"
else
    print_warn "cargo-tarpaulin not found. Installing..."
    if cargo install cargo-tarpaulin; then
        run_test "Code coverage" "cargo tarpaulin --all-features --workspace --timeout 120 --out json --output-dir coverage"
    else
        print_error "Failed to install cargo-tarpaulin"
        ((TESTS_FAILED++))
        FAILED_TESTS+=("Code coverage")
    fi
fi

# Container tests (if Docker available)
if command_exists docker; then
    print_section "Container Tests"
    run_test "Container isolation tests" "cargo test test_container_isolation"
else
    print_warn "Skipping container tests - Docker not available"
fi

# Build tests for different targets
print_section "Build Tests"

run_test "Debug build" "cargo build --workspace"
run_test "Release build" "cargo build --workspace --release"

# Feature flag tests
print_section "Feature Tests"

run_test "No default features build" "cargo build --no-default-features"
run_test "All features build" "cargo build --all-features"

# Generate final report
print_section "Test Results Summary"

TOTAL_TESTS=$((TESTS_PASSED + TESTS_FAILED))

printf "\n"
printf "Total Tests: %d\n" "$TOTAL_TESTS"
printf "${GREEN}Passed: %d${NC}\n" "$TESTS_PASSED"
printf "${RED}Failed: %d${NC}\n" "$TESTS_FAILED"

if [ ${#FAILED_TESTS[@]} -gt 0 ]; then
    printf "\n${RED}Failed Tests:${NC}\n"
    for test in "${FAILED_TESTS[@]}"; do
        printf "  - %s\n" "$test"
    done
fi

# Calculate success rate
if [ "$TOTAL_TESTS" -gt 0 ]; then
    SUCCESS_RATE=$((TESTS_PASSED * 100 / TOTAL_TESTS))
    printf "\nSuccess Rate: %d%%\n" "$SUCCESS_RATE"
    
    if [ "$SUCCESS_RATE" -ge 95 ]; then
        printf "${GREEN}🎉 Excellent! Test suite passed with flying colors!${NC}\n"
    elif [ "$SUCCESS_RATE" -ge 80 ]; then
        printf "${YELLOW}⚠️  Good, but some issues need attention.${NC}\n"
    else
        printf "${RED}❌ Test suite has significant failures that need to be addressed.${NC}\n"
    fi
fi

# Coverage report (if available)
if [ -f "coverage/tarpaulin-report.json" ]; then
    print_section "Coverage Summary"
    
    # Extract coverage percentage from tarpaulin report
    COVERAGE=$(jq -r '.coverage' coverage/tarpaulin-report.json 2>/dev/null || echo "N/A")
    
    if [ "$COVERAGE" != "N/A" ]; then
        printf "Code Coverage: %.2f%%\n" "$COVERAGE"
        
        if (( $(echo "$COVERAGE >= 80" | bc -l) )); then
            printf "${GREEN}✅ Good code coverage${NC}\n"
        elif (( $(echo "$COVERAGE >= 60" | bc -l) )); then
            printf "${YELLOW}⚠️  Moderate code coverage${NC}\n"
        else
            printf "${RED}❌ Low code coverage${NC}\n"
        fi
    fi
fi

# Generate test report
cat > test-report.md << EOF
# OpenCode-RS Test Report

**Date:** $(date)
**Total Tests:** $TOTAL_TESTS
**Passed:** $TESTS_PASSED
**Failed:** $TESTS_FAILED
**Success Rate:** ${SUCCESS_RATE:-0}%

## Failed Tests
$(for test in "${FAILED_TESTS[@]}"; do echo "- $test"; done)

## Coverage
$([ -f "coverage/tarpaulin-report.json" ] && echo "Code Coverage: ${COVERAGE:-N/A}%" || echo "Coverage data not available")

## Environment
- OS: $(uname -s) $(uname -r)
- Rust: $(rustc --version)
- Cargo: $(cargo --version)
- Node: $(node --version 2>/dev/null || echo "Not available")
- Docker: $(docker --version 2>/dev/null || echo "Not available")
EOF

print_info "Test report saved to test-report.md"

# Exit with error code if any tests failed
if [ "$TESTS_FAILED" -gt 0 ]; then
    print_error "Test suite completed with failures"
    exit 1
else
    print_info "🎉 All tests passed successfully!"
    exit 0
fi