#!/bin/bash

# Coverage monitoring script for opencode-rs
# Ensures 100% test coverage across all crates

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
MIN_COVERAGE=100.0
COVERAGE_DIR="coverage"
WORKSPACE_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

cd "$WORKSPACE_ROOT"

# Ensure coverage directory exists
mkdir -p "$COVERAGE_DIR"

echo -e "${BLUE}🚀 Running comprehensive test coverage analysis for opencode-rs${NC}"
echo "====================================================================="

# Install tarpaulin if not present
if ! command -v cargo-tarpaulin &> /dev/null; then
    echo -e "${YELLOW}⚠️  Installing cargo-tarpaulin...${NC}"
    cargo install cargo-tarpaulin
fi

# Clean previous coverage data
echo -e "${BLUE}🧹 Cleaning previous coverage data...${NC}"
rm -rf "$COVERAGE_DIR"/*
cargo clean

# Run tests with coverage for all crates
echo -e "${BLUE}📊 Running coverage analysis for all crates...${NC}"
cargo tarpaulin \
    --workspace \
    --all-features \
    --timeout 300 \
    --out Html \
    --out Json \
    --out Lcov \
    --output-dir "$COVERAGE_DIR/total" \
    --skip-clean \
    --ignore-panics \
    --ignore-tests \
    --exclude-files "target/*" \
    --exclude-files "*/tests/*" \
    --exclude-files "*/test_utils/*" \
    --verbose

# Run coverage for individual crates
echo -e "${BLUE}📊 Running coverage analysis for individual crates...${NC}"

# Core crate
echo -e "${YELLOW}  Analyzing opencode_core crate...${NC}"
cargo tarpaulin \
    --package opencode_core \
    --all-features \
    --timeout 300 \
    --out Html \
    --out Json \
    --output-dir "$COVERAGE_DIR/core" \
    --skip-clean \
    --ignore-panics \
    --ignore-tests \
    --exclude-files "target/*" \
    --exclude-files "*/tests/*" \
    --exclude-files "*/test_utils/*"

# Parse coverage results
echo -e "${BLUE}📈 Analyzing coverage results...${NC}"

# Function to extract coverage percentage from JSON
extract_coverage() {
    local json_file="$1"
    if [[ -f "$json_file" ]]; then
        python3 -c "
import json, sys
try:
    with open('$json_file', 'r') as f:
        data = json.load(f)
    covered = data['files'].values() if 'files' in data else []
    total_lines = sum(file_data.get('summary', {}).get('lines', {}).get('total', 0) for file_data in covered)
    covered_lines = sum(file_data.get('summary', {}).get('lines', {}).get('covered', 0) for file_data in covered)
    percentage = (covered_lines / total_lines * 100) if total_lines > 0 else 0
    print(f'{percentage:.2f}')
except Exception as e:
    print('0.00')
"
    else
        echo "0.00"
    fi
}

# Extract coverage percentages
TOTAL_COVERAGE=$(extract_coverage "$COVERAGE_DIR/total/tarpaulin-report.json")
CORE_COVERAGE=$(extract_coverage "$COVERAGE_DIR/core/tarpaulin-report.json")

echo "====================================================================="
echo -e "${BLUE}📊 COVERAGE SUMMARY${NC}"
echo "====================================================================="
echo -e "Total Workspace Coverage: ${GREEN}${TOTAL_COVERAGE}%${NC}"
echo -e "Core Crate Coverage:      ${GREEN}${CORE_COVERAGE}%${NC}"
echo "====================================================================="

# Check if coverage meets minimum requirements
coverage_check() {
    local coverage="$1"
    local name="$2"
    local min_required="$3"
    
    if (( $(echo "$coverage >= $min_required" | bc -l) )); then
        echo -e "${GREEN}✅ $name coverage: $coverage% (>= $min_required%)${NC}"
        return 0
    else
        echo -e "${RED}❌ $name coverage: $coverage% (< $min_required%)${NC}"
        return 1
    fi
}

# Validate coverage requirements
echo -e "${BLUE}🎯 Validating coverage requirements...${NC}"
COVERAGE_PASSED=true

if ! coverage_check "$TOTAL_COVERAGE" "Total workspace" "$MIN_COVERAGE"; then
    COVERAGE_PASSED=false
fi

if ! coverage_check "$CORE_COVERAGE" "Core crate" "$MIN_COVERAGE"; then
    COVERAGE_PASSED=false
fi

# Generate detailed coverage report
echo -e "${BLUE}📋 Generating detailed coverage reports...${NC}"

# Create a summary HTML file
cat > "$COVERAGE_DIR/index.html" << EOF
<!DOCTYPE html>
<html>
<head>
    <title>OpenCode-RS Coverage Summary</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        .header { background: #f0f0f0; padding: 20px; border-radius: 5px; }
        .metric { display: inline-block; margin: 10px; padding: 15px; border-radius: 5px; }
        .good { background: #d4edda; color: #155724; }
        .bad { background: #f8d7da; color: #721c24; }
        .coverage-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(300px, 1fr)); gap: 20px; margin: 20px 0; }
        .coverage-card { border: 1px solid #ddd; padding: 15px; border-radius: 5px; }
    </style>
</head>
<body>
    <div class="header">
        <h1>🚀 OpenCode-RS Coverage Summary</h1>
        <p>Generated on: $(date)</p>
    </div>
    
    <div class="coverage-grid">
        <div class="coverage-card">
            <h3>📊 Total Workspace Coverage</h3>
            <div class="metric $(if (( $(echo "$TOTAL_COVERAGE >= $MIN_COVERAGE" | bc -l) )); then echo "good"; else echo "bad"; fi)">
                ${TOTAL_COVERAGE}%
            </div>
            <p><a href="total/tarpaulin-report.html">View Detailed Report</a></p>
        </div>
        
        <div class="coverage-card">
            <h3>🎯 Core Crate Coverage</h3>
            <div class="metric $(if (( $(echo "$CORE_COVERAGE >= $MIN_COVERAGE" | bc -l) )); then echo "good"; else echo "bad"; fi)">
                ${CORE_COVERAGE}%
            </div>
            <p><a href="core/tarpaulin-report.html">View Detailed Report</a></p>
        </div>
    </div>
    
    <h2>📈 Coverage Goals</h2>
    <ul>
        <li>Minimum Required Coverage: ${MIN_COVERAGE}%</li>
        <li>Target Coverage: 100%</li>
        <li>Coverage includes all production code paths</li>
        <li>Excludes test utilities and generated code</li>
    </ul>
    
    <h2>🔗 Quick Links</h2>
    <ul>
        <li><a href="total/tarpaulin-report.html">Complete Workspace Coverage Report</a></li>
        <li><a href="core/tarpaulin-report.html">Core Crate Coverage Report</a></li>
        <li><a href="total/lcov.info">LCOV Data (Total)</a></li>
        <li><a href="core/lcov.info">LCOV Data (Core)</a></li>
    </ul>
</body>
</html>
EOF

# Generate coverage badge
echo -e "${BLUE}🏆 Generating coverage badges...${NC}"
if (( $(echo "$TOTAL_COVERAGE >= 95" | bc -l) )); then
    BADGE_COLOR="brightgreen"
elif (( $(echo "$TOTAL_COVERAGE >= 80" | bc -l) )); then
    BADGE_COLOR="yellow"
else
    BADGE_COLOR="red"
fi

# Create badge SVG
cat > "$COVERAGE_DIR/badge.svg" << EOF
<svg xmlns="http://www.w3.org/2000/svg" width="104" height="20">
    <linearGradient id="b" x2="0" y2="100%">
        <stop offset="0" stop-color="#bbb" stop-opacity=".1"/>
        <stop offset="1" stop-opacity=".1"/>
    </linearGradient>
    <mask id="a">
        <rect width="104" height="20" rx="3" fill="#fff"/>
    </mask>
    <g mask="url(#a)">
        <path fill="#555" d="M0 0h63v20H0z"/>
        <path fill="$BADGE_COLOR" d="M63 0h41v20H63z"/>
        <path fill="url(#b)" d="M0 0h104v20H0z"/>
    </g>
    <g fill="#fff" text-anchor="middle" font-family="DejaVu Sans,Verdana,Geneva,sans-serif" font-size="11">
        <text x="31.5" y="15" fill="#010101" fill-opacity=".3">coverage</text>
        <text x="31.5" y="14">coverage</text>
        <text x="82.5" y="15" fill="#010101" fill-opacity=".3">${TOTAL_COVERAGE}%</text>
        <text x="82.5" y="14">${TOTAL_COVERAGE}%</text>
    </g>
</svg>
EOF

# Create coverage trend log
TREND_FILE="$COVERAGE_DIR/trend.log"
echo "$(date +'%Y-%m-%d %H:%M:%S'),${TOTAL_COVERAGE},${CORE_COVERAGE}" >> "$TREND_FILE"

# Display final results
echo "====================================================================="
if $COVERAGE_PASSED; then
    echo -e "${GREEN}🎉 SUCCESS: All coverage requirements met!${NC}"
    echo -e "${GREEN}✅ Total coverage: ${TOTAL_COVERAGE}% (>= ${MIN_COVERAGE}%)${NC}"
    echo -e "${GREEN}✅ Core coverage: ${CORE_COVERAGE}% (>= ${MIN_COVERAGE}%)${NC}"
else
    echo -e "${RED}❌ FAILURE: Coverage requirements not met!${NC}"
    echo -e "${RED}Target: ${MIN_COVERAGE}% minimum coverage${NC}"
    echo -e "${YELLOW}💡 Please add tests to increase coverage${NC}"
fi

echo "====================================================================="
echo -e "${BLUE}📁 Coverage reports generated in: ${COVERAGE_DIR}/${NC}"
echo -e "${BLUE}🌐 Open coverage/index.html in a browser to view results${NC}"
echo "====================================================================="

# Exit with appropriate code
if $COVERAGE_PASSED; then
    exit 0
else
    exit 1
fi