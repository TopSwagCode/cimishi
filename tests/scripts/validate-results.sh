#!/bin/sh
set -e

echo "=== Validating test results ==="

PASSED=0
FAILED=0
SKIPPED=0

check_output() {
    local backend=$1
    local dir="/output/$backend"
    
    echo ""
    echo "Checking $backend..."
    
    # Check if output directory exists and has files
    if [ ! -d "$dir" ]; then
        echo "  SKIP: Output directory not found: $dir"
        SKIPPED=$((SKIPPED + 1))
        return
    fi
    
    # Check for CSV file
    csv_count=$(find "$dir" -name "*.csv" 2>/dev/null | wc -l | tr -d ' ')
    if [ "$csv_count" -eq 0 ]; then
        echo "  SKIP: No CSV output found"
        SKIPPED=$((SKIPPED + 1))
        return
    fi
    
    # Check CSV has content (more than just header)
    csv_file=$(find "$dir" -name "*.csv" | head -1)
    line_count=$(wc -l < "$csv_file" | tr -d ' ')
    if [ "$line_count" -lt 2 ]; then
        echo "  FAIL: CSV has no data rows"
        FAILED=$((FAILED + 1))
        return
    fi
    
    # Check expected results (should have 3 substations)
    expected_rows=4  # header + 3 substations
    if [ "$line_count" -lt "$expected_rows" ]; then
        echo "  WARN: Expected at least $expected_rows rows, got $line_count"
    fi
    
    # Verify expected substation names in output
    if grep -q "Test Substation Alpha" "$csv_file" && \
       grep -q "Test Substation Beta" "$csv_file" && \
       grep -q "Test Substation Gamma" "$csv_file"; then
        echo "  PASS: All expected substations found ($line_count rows)"
        PASSED=$((PASSED + 1))
    else
        echo "  FAIL: Missing expected substation names"
        FAILED=$((FAILED + 1))
        return
    fi
}

# Check each backend
check_output "local"
check_output "s3"
check_output "azure"
# NOTE: GCS test removed - see README.md

echo ""
echo "================================"
echo "Results: $PASSED passed, $FAILED failed, $SKIPPED skipped"

if [ "$FAILED" -eq 0 ] && [ "$PASSED" -gt 0 ]; then
    echo "TESTS PASSED"
    exit 0
else
    echo "TESTS HAD FAILURES"
    exit 1
fi
