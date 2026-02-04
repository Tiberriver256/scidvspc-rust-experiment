#!/bin/bash
# Script to generate test fixtures from SCID games

set -e

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
FIXTURE_GEN_DIR="$PROJECT_ROOT/tools/fixture_generator"
OUTPUT_DIR="$PROJECT_ROOT/rust-port/tests/fixtures"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== SCID Test Fixture Generator ===${NC}"

# Step 1: Build the fixture generator
echo -e "\n${YELLOW}Building fixture generator...${NC}"
if [ ! -d "$FIXTURE_GEN_DIR" ]; then
    echo -e "${RED}Error: Fixture generator directory not found${NC}"
    echo "Expected: $FIXTURE_GEN_DIR"
    exit 1
fi

cd "$FIXTURE_GEN_DIR"
make clean
make

if [ $? -ne 0 ]; then
    echo -e "${RED}Failed to build fixture generator${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Fixture generator built successfully${NC}"

# Step 2: Create output directory structure
echo -e "\n${YELLOW}Creating output directory structure...${NC}"
mkdir -p "$OUTPUT_DIR"/{simple_games,with_variations,with_comments,with_nags,special_cases,all_formats,regression}

# Step 3: Generate test fixtures
echo -e "\n${YELLOW}Generating test fixtures...${NC}"

# Simple games (no variations or comments)
echo "  - Simple games..."
./fixture_generator \
    --type simple \
    --count 20 \
    --output "$OUTPUT_DIR/simple_games"

# Games with variations
echo "  - Games with variations..."
./fixture_generator \
    --type variations \
    --count 15 \
    --output "$OUTPUT_DIR/with_variations"

# Games with comments
echo "  - Games with comments..."
./fixture_generator \
    --type comments \
    --count 15 \
    --output "$OUTPUT_DIR/with_comments"

# Games with NAG annotations
echo "  - Games with NAGs..."
./fixture_generator \
    --type nags \
    --count 10 \
    --output "$OUTPUT_DIR/with_nags"

# Special cases
echo "  - Special cases..."
./fixture_generator \
    --type special \
    --count 25 \
    --output "$OUTPUT_DIR/special_cases"

# All formats (HTML, LaTeX, etc.)
echo "  - All formats..."
./fixture_generator \
    --type formats \
    --count 10 \
    --output "$OUTPUT_DIR/all_formats"

echo -e "${GREEN}✓ Test fixtures generated successfully${NC}"

# Step 4: Validate fixtures
echo -e "\n${YELLOW}Validating generated fixtures...${NC}"
FIXTURE_COUNT=$(find "$OUTPUT_DIR" -name "*.json" | wc -l)
echo "  Total fixtures generated: $FIXTURE_COUNT"

# Check that each fixture is valid JSON
INVALID_COUNT=0
for fixture in $(find "$OUTPUT_DIR" -name "*.json"); do
    if ! python3 -m json.tool "$fixture" > /dev/null 2>&1; then
        echo -e "${RED}  Invalid JSON: $fixture${NC}"
        INVALID_COUNT=$((INVALID_COUNT + 1))
    fi
done

if [ $INVALID_COUNT -eq 0 ]; then
    echo -e "${GREEN}✓ All fixtures are valid JSON${NC}"
else
    echo -e "${RED}✗ Found $INVALID_COUNT invalid JSON files${NC}"
    exit 1
fi

# Step 5: Generate summary report
echo -e "\n${YELLOW}Generating summary report...${NC}"
cat > "$OUTPUT_DIR/README.md" << 'EOF'
# Test Fixtures for PGN Conversion

This directory contains test fixtures for validating the SCID to PGN conversion
functionality in both C++ and Rust implementations.

## Directory Structure

- `simple_games/` - Basic games without variations or comments
- `with_variations/` - Games with single and nested variations
- `with_comments/` - Games with move comments
- `with_nags/` - Games with NAG annotations
- `special_cases/` - Edge cases and unusual situations
- `all_formats/` - Games in different output formats (HTML, LaTeX, etc.)
- `regression/` - Tests for specific bug fixes

## Fixture Format

Each fixture is a JSON file with the following structure:

```json
{
  "name": "unique_test_name",
  "description": "Human-readable description",
  "input": {
    // Game data
  },
  "expected_pgn": "...",
  "options": {
    // Conversion options
  }
}
```

## Usage

### In Rust Tests

```rust
use std::path::Path;
use serde_json;

#[test]
fn test_pgn_conversion() {
    let fixture = load_fixture("simple_games/basic_game_001.json");
    let result = convert_to_pgn(&fixture.input, &fixture.options);
    assert_eq!(result, fixture.expected_pgn);
}
```

### Cross-Validation

```bash
cargo run --bin cross-validator -- --fixtures tests/fixtures/
```

## Regenerating Fixtures

```bash
cd scripts
./generate_test_fixtures.sh
```

This will rebuild the fixture generator and regenerate all test fixtures.
EOF

echo -e "${GREEN}✓ Summary report generated${NC}"

# Final summary
echo -e "\n${GREEN}=== Summary ===${NC}"
echo "Fixtures directory: $OUTPUT_DIR"
echo "Total fixtures: $FIXTURE_COUNT"
echo ""
echo "Next steps:"
echo "  1. Review the generated fixtures"
echo "  2. Initialize the Rust project: cd rust-port && cargo init"
echo "  3. Run the tests: cd rust-port && cargo test"
echo ""
echo -e "${GREEN}Done!${NC}"
