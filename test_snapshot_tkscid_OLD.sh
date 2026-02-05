#!/bin/bash
# Randomized snapshot test - compare Rust output to C++ oracle

set -e

# Available databases with game counts
declare -A DATABASES
DATABASES[endings]=67
DATABASES[matein1]=1212
DATABASES[matein2]=15623
DATABASES[matein3]=7862
DATABASES[matein4andmore]=3417
DATABASES[tactics]=928

# Pick a random database
DB_NAMES=(${!DATABASES[@]})
RANDOM_DB=${DB_NAMES[$RANDOM % ${#DB_NAMES[@]}]}
MAX_GAMES=${DATABASES[$RANDOM_DB]}

# Pick 3 random game numbers
GAME1=$((1 + RANDOM % MAX_GAMES))
GAME2=$((1 + RANDOM % MAX_GAMES))
GAME3=$((1 + RANDOM % MAX_GAMES))

echo "Running randomized snapshot test..."
echo "===================================="
echo "Database: bases/$RANDOM_DB ($MAX_GAMES games)"
echo "Games: $GAME1, $GAME2, $GAME3"
echo ""

# Generate oracle (C++)
timeout 30 ./tkscid extract.tcl bases/$RANDOM_DB $GAME1 $GAME2 $GAME3 2>/dev/null > /tmp/cpp_output.txt || {
    echo "❌ FAIL: C++ oracle timed out or crashed"
    exit 1
}

# Generate Rust output
cd rust-port && cargo run --example rust_extractor ../bases/$RANDOM_DB $GAME1 $GAME2 $GAME3 2>/dev/null > /tmp/rust_output.txt || {
    echo "❌ FAIL: Rust extractor crashed"
    exit 1
}

# Compare
if diff -u /tmp/cpp_output.txt /tmp/rust_output.txt > /tmp/diff.txt; then
    echo "✅ PASS: Outputs match perfectly!"
    echo ""
    echo "Sample output (first game):"
    head -15 /tmp/cpp_output.txt
    exit 0
else
    echo "❌ FAIL: Outputs differ"
    echo ""
    echo "Differences:"
    head -100 /tmp/diff.txt
    echo ""
    echo "Full diff saved to /tmp/diff.txt"
    echo "C++ output: /tmp/cpp_output.txt"
    echo "Rust output: /tmp/rust_output.txt"
    exit 1
fi
