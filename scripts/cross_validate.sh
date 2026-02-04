#!/bin/bash
# Cross-validator: Compares Rust output against C++ baseline
# This runs the same test cases through both implementations

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${GREEN}=== Cross-Validator: Rust vs C++ ===${NC}\n"

# Check prerequisites
if [ ! -d "$PROJECT_ROOT/rust-port/tests/fixtures" ]; then
    echo -e "${RED}Error: No fixtures found.${NC}"
    echo "Run ./scripts/generate_test_fixtures.sh first"
    exit 1
fi

# Build Rust implementation
echo -e "${YELLOW}Building Rust implementation...${NC}"
cd "$PROJECT_ROOT/rust-port"
cargo build --release 2>&1 | grep -v "Compiling" | grep -v "Finished" || true
echo -e "${GREEN}✓ Rust build complete${NC}\n"

# Run tests and capture results
echo -e "${YELLOW}Running cross-validation tests...${NC}\n"

RESULTS_FILE="/tmp/scid_cross_validation_results.txt"
cargo test test_fixtures -- --nocapture 2>&1 | tee "$RESULTS_FILE"

# Parse results
PASSED=$(grep -o "test result: ok. [0-9]* passed" "$RESULTS_FILE" | grep -o "[0-9]*" | head -1)
FAILED=$(grep -o "[0-9]* failed" "$RESULTS_FILE" | grep -o "[0-9]*" | head -1)

if [ -z "$PASSED" ]; then PASSED=0; fi
if [ -z "$FAILED" ]; then FAILED=0; fi

TOTAL=$((PASSED + FAILED))

echo ""
echo -e "${GREEN}╔═══════════════════════════════════════╗${NC}"
echo -e "${GREEN}║     Cross-Validation Summary          ║${NC}"
echo -e "${GREEN}╚═══════════════════════════════════════╝${NC}"
echo ""

if [ $TOTAL -eq 0 ]; then
    echo -e "${YELLOW}No fixture tests found.${NC}"
    echo "This is expected if fixtures haven't been generated yet."
    echo ""
    echo "To generate fixtures:"
    echo "  ./scripts/generate_test_fixtures.sh"
else
    echo -e "Total tests:     ${TOTAL}"
    echo -e "Passed:          ${GREEN}${PASSED}${NC}"
    echo -e "Failed:          ${RED}${FAILED}${NC}"
    
    PERCENT=0
    if [ $TOTAL -gt 0 ]; then
        PERCENT=$((PASSED * 100 / TOTAL))
    fi
    echo -e "Success rate:    ${PERCENT}%"
    echo ""
    
    if [ $FAILED -eq 0 ]; then
        echo -e "${GREEN}🎉 All tests passed! Rust matches C++ behavior.${NC}"
    else
        echo -e "${YELLOW}⚠️  Some tests failed. Check output above for details.${NC}"
        echo ""
        echo "Failed tests indicate differences between Rust and C++ output."
        echo "Review the test output to see what needs to be implemented."
    fi
fi

echo ""
echo "Detailed results saved to: $RESULTS_FILE"
echo ""

# Generate a progress report
REPORT_FILE="$PROJECT_ROOT/test-progress.md"
cat > "$REPORT_FILE" << EOF
# Test Progress Report

Generated: $(date)

## Cross-Validation Results

- **Total Tests**: $TOTAL
- **Passed**: $PASSED ✅
- **Failed**: $FAILED ❌
- **Success Rate**: ${PERCENT}%

## Status by Category

EOF

# Try to extract category information from test names
if [ $TOTAL -gt 0 ]; then
    cat >> "$REPORT_FILE" << EOF
Run \`./scripts/generate_test_fixtures.sh\` to create test fixtures.
Then run this script again to see category breakdown.

EOF
fi

cat >> "$REPORT_FILE" << EOF
## Next Steps

EOF

if [ $FAILED -gt 0 ]; then
    cat >> "$REPORT_FILE" << EOF
1. Review failed tests in the output above
2. Implement missing functionality in Rust
3. Re-run this script to validate fixes
4. Repeat until all tests pass

EOF
elif [ $TOTAL -eq 0 ]; then
    cat >> "$REPORT_FILE" << EOF
1. Generate test fixtures: \`./scripts/generate_test_fixtures.sh\`
2. Run cross-validation again to measure progress

EOF
else
    cat >> "$REPORT_FILE" << EOF
All tests passing! 🎉

Consider:
1. Adding more edge case tests
2. Performance optimization
3. Documentation improvements
4. API refinement

EOF
fi

cat >> "$REPORT_FILE" << EOF
## Implementation Status

- [x] Core types (Square, Piece, Move)
- [x] Game structure
- [x] PGN tags
- [ ] Move formatting (SAN notation)
- [ ] Variation handling
- [ ] Comment formatting
- [ ] NAG annotations
- [ ] HTML/LaTeX output

## Resources

- Main documentation: docs/rust-port-tdd-approach.md
- Rust README: rust-port/README.md
- Quick help: ./scripts/help.sh
EOF

echo -e "${GREEN}Progress report saved to: test-progress.md${NC}"
echo ""
