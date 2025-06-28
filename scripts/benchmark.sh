#!/bin/bash
set -e

# Performance benchmarking script for OpenCode-RS

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

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

# Check if criterion benchmarks are available
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

print_section "OpenCode-RS Performance Benchmarks"

# Create benchmark directory if it doesn't exist
mkdir -p benchmarks/results

print_info "Starting performance benchmarks..."

# Build in release mode first
print_section "Building Release Version"
print_info "Building optimized release version..."
cargo build --release --all-features

print_section "CLI Performance Benchmarks"

# Measure CLI startup time
print_info "Measuring CLI startup time..."
START_TIME_FILE=$(mktemp)

for i in {1..10}; do
    TIME_START=$(date +%s%N)
    ./target/release/opencode --version > /dev/null 2>&1
    TIME_END=$(date +%s%N)
    ELAPSED=$((TIME_END - TIME_START))
    echo "$ELAPSED" >> "$START_TIME_FILE"
done

AVERAGE_STARTUP=$(awk '{ sum += $1; n++ } END { if (n > 0) print sum / n; }' "$START_TIME_FILE")
AVERAGE_STARTUP_MS=$(echo "scale=2; $AVERAGE_STARTUP / 1000000" | bc)

printf "CLI Startup Time: %.2f ms (average of 10 runs)\n" "$AVERAGE_STARTUP_MS"

# Measure CLI help command performance
print_info "Measuring CLI help command performance..."
HELP_TIME_FILE=$(mktemp)

for i in {1..5}; do
    TIME_START=$(date +%s%N)
    ./target/release/opencode --help > /dev/null 2>&1
    TIME_END=$(date +%s%N)
    ELAPSED=$((TIME_END - TIME_START))
    echo "$ELAPSED" >> "$HELP_TIME_FILE"
done

AVERAGE_HELP=$(awk '{ sum += $1; n++ } END { if (n > 0) print sum / n; }' "$HELP_TIME_FILE")
AVERAGE_HELP_MS=$(echo "scale=2; $AVERAGE_HELP / 1000000" | bc)

printf "CLI Help Command: %.2f ms (average of 5 runs)\n" "$AVERAGE_HELP_MS"

# Cleanup temp files
rm -f "$START_TIME_FILE" "$HELP_TIME_FILE"

print_section "Memory Usage Benchmarks"

# Measure memory usage
print_info "Measuring memory usage..."

if command_exists valgrind; then
    print_info "Running memory analysis with Valgrind..."
    valgrind --tool=massif --massif-out-file=benchmarks/results/massif.out \
        ./target/release/opencode --version > /dev/null 2>&1
    
    if command_exists ms_print; then
        ms_print benchmarks/results/massif.out > benchmarks/results/memory-report.txt
        PEAK_MEMORY=$(grep "Peak" benchmarks/results/memory-report.txt | head -1)
        print_info "Memory usage: $PEAK_MEMORY"
    fi
else
    print_warn "Valgrind not available. Using basic memory measurement..."
    
    # Use time command to get basic memory info
    /usr/bin/time -l ./target/release/opencode --version > benchmarks/results/memory-basic.txt 2>&1 || \
    /usr/bin/time -v ./target/release/opencode --version > benchmarks/results/memory-basic.txt 2>&1 || \
    time ./target/release/opencode --version > benchmarks/results/memory-basic.txt 2>&1
    
    if [ -f benchmarks/results/memory-basic.txt ]; then
        print_info "Basic memory usage recorded in benchmarks/results/memory-basic.txt"
    fi
fi

print_section "Core Library Benchmarks"

# Run Rust benchmarks if available
if [ -f "benches/criterion.rs" ] || find benches -name "*.rs" 2>/dev/null | grep -q .; then
    print_info "Running Criterion benchmarks..."
    cargo bench --all-features -- --output-format json --output-file benchmarks/results/criterion.json
    print_info "Criterion benchmarks completed"
else
    print_warn "No Criterion benchmarks found. Creating sample benchmark..."
    
    # Create a simple benchmark test
    cargo test test_performance_benchmarks --release -- --nocapture | tee benchmarks/results/simple-benchmarks.txt
fi

print_section "Compilation Time Benchmarks"

print_info "Measuring compilation times..."

# Clean build for accurate timing
cargo clean

# Measure clean build time
print_info "Measuring clean build time..."
CLEAN_BUILD_START=$(date +%s)
cargo build --release --all-features > benchmarks/results/build-output.txt 2>&1
CLEAN_BUILD_END=$(date +%s)
CLEAN_BUILD_TIME=$((CLEAN_BUILD_END - CLEAN_BUILD_START))

printf "Clean build time: %d seconds\n" "$CLEAN_BUILD_TIME"

# Measure incremental build time (no changes)
print_info "Measuring incremental build time..."
INCREMENTAL_START=$(date +%s)
cargo build --release --all-features > /dev/null 2>&1
INCREMENTAL_END=$(date +%s)
INCREMENTAL_TIME=$((INCREMENTAL_END - INCREMENTAL_START))

printf "Incremental build time: %d seconds\n" "$INCREMENTAL_TIME"

print_section "Frontend Performance Benchmarks"

if [ -d "crates/opencode-gui" ] && command_exists pnpm; then
    cd crates/opencode-gui
    
    print_info "Measuring frontend build time..."
    FRONTEND_BUILD_START=$(date +%s)
    pnpm build > ../../benchmarks/results/frontend-build.txt 2>&1
    FRONTEND_BUILD_END=$(date +%s)
    FRONTEND_BUILD_TIME=$((FRONTEND_BUILD_END - FRONTEND_BUILD_START))
    
    printf "Frontend build time: %d seconds\n" "$FRONTEND_BUILD_TIME"
    
    # Measure bundle size
    if [ -d "dist" ]; then
        BUNDLE_SIZE=$(du -sh dist | cut -f1)
        print_info "Frontend bundle size: $BUNDLE_SIZE"
    fi
    
    cd ../..
else
    print_warn "Frontend benchmarks skipped - GUI directory not found or pnpm not available"
fi

print_section "Load Testing"

# Simple load test for CLI
print_info "Running CLI load test..."
LOAD_TEST_START=$(date +%s)

# Run CLI commands in parallel
for i in {1..20}; do
    (./target/release/opencode --version > /dev/null 2>&1) &
done

# Wait for all background jobs to complete
wait

LOAD_TEST_END=$(date +%s)
LOAD_TEST_TIME=$((LOAD_TEST_END - LOAD_TEST_START))

printf "CLI load test (20 parallel executions): %d seconds\n" "$LOAD_TEST_TIME"

print_section "Binary Size Analysis"

# Measure binary sizes
print_info "Analyzing binary sizes..."

CLI_SIZE=$(ls -lh target/release/opencode | awk '{print $5}')
print_info "CLI binary size: $CLI_SIZE"

# Strip symbols and measure again
cp target/release/opencode target/release/opencode-stripped
strip target/release/opencode-stripped 2>/dev/null || true
CLI_STRIPPED_SIZE=$(ls -lh target/release/opencode-stripped | awk '{print $5}')
print_info "CLI binary size (stripped): $CLI_STRIPPED_SIZE"

# Analyze with bloaty if available
if command_exists bloaty; then
    print_info "Running detailed binary analysis..."
    bloaty target/release/opencode > benchmarks/results/binary-analysis.txt 2>&1
    print_info "Binary analysis saved to benchmarks/results/binary-analysis.txt"
fi

print_section "Performance Summary Report"

# Generate performance report
cat > benchmarks/results/performance-report.md << EOF
# OpenCode-RS Performance Report

**Date:** $(date)
**System:** $(uname -a)
**Rust Version:** $(rustc --version)

## CLI Performance
- Startup Time: ${AVERAGE_STARTUP_MS:-N/A} ms
- Help Command: ${AVERAGE_HELP_MS:-N/A} ms
- Load Test (20 parallel): ${LOAD_TEST_TIME} seconds

## Build Performance
- Clean Build Time: ${CLEAN_BUILD_TIME} seconds
- Incremental Build Time: ${INCREMENTAL_TIME} seconds
$([ -n "$FRONTEND_BUILD_TIME" ] && echo "- Frontend Build Time: ${FRONTEND_BUILD_TIME} seconds")

## Binary Sizes
- CLI Binary: ${CLI_SIZE}
- CLI Binary (stripped): ${CLI_STRIPPED_SIZE}
$([ -n "$BUNDLE_SIZE" ] && echo "- Frontend Bundle: ${BUNDLE_SIZE}")

## Memory Usage
$([ -f "benchmarks/results/memory-report.txt" ] && echo "- See benchmarks/results/memory-report.txt for detailed analysis" || echo "- Basic memory usage logged")

## Notes
- All benchmarks run on: $(hostname)
- CPU: $(sysctl -n machdep.cpu.brand_string 2>/dev/null || grep "model name" /proc/cpuinfo | head -1 | cut -d: -f2 | xargs || echo "Unknown")
- RAM: $(sysctl -n hw.memsize 2>/dev/null | awk '{print $1/1024/1024/1024 " GB"}' || grep "MemTotal" /proc/meminfo | awk '{print $2/1024/1024 " GB"}' || echo "Unknown")

EOF

# Performance thresholds and warnings
print_section "Performance Analysis"

# Check if performance metrics meet expectations
if [ -n "$AVERAGE_STARTUP_MS" ]; then
    if (( $(echo "$AVERAGE_STARTUP_MS > 100" | bc -l) )); then
        print_warn "CLI startup time is high (${AVERAGE_STARTUP_MS}ms > 100ms)"
    else
        print_info "CLI startup time is acceptable (${AVERAGE_STARTUP_MS}ms)"
    fi
fi

if [ "$CLEAN_BUILD_TIME" -gt 300 ]; then
    print_warn "Clean build time is high (${CLEAN_BUILD_TIME}s > 300s)"
else
    print_info "Clean build time is acceptable (${CLEAN_BUILD_TIME}s)"
fi

if [ "$INCREMENTAL_TIME" -gt 10 ]; then
    print_warn "Incremental build time is high (${INCREMENTAL_TIME}s > 10s)"
else
    print_info "Incremental build time is good (${INCREMENTAL_TIME}s)"
fi

print_info "Performance report saved to benchmarks/results/performance-report.md"
print_info "All benchmark results saved to benchmarks/results/"

# List all generated files
print_section "Generated Files"
find benchmarks/results -type f -exec ls -lh {} \; | while read -r line; do
    print_info "$line"
done

print_info "🎯 Performance benchmarking completed!"