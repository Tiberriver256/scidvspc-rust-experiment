#!/bin/bash
# Randomized snapshot test - compare Rust output to C++ oracle (using tcscid)

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

ALL_PASS=true

for GNUM in $GAME1 $GAME2 $GAME3; do
    # Generate C++ baseline using tcscid
    # sc_game pgn includes trailing newline, so we don't add one before ###
    ./tcscid << EOF > /tmp/cpp_$GNUM.txt 2>/dev/null
sc_base open bases/$RANDOM_DB
sc_game load $GNUM
puts -nonewline "### GAME $GNUM ###\n"
puts [sc_game pgn]
puts -nonewline "### END GAME $GNUM ###\n\n"
exit
EOF

    # Generate Rust output
    cargo run --example rust_extractor --manifest-path rust-port/Cargo.toml bases/$RANDOM_DB $GNUM > /tmp/rust_$GNUM.txt 2>/dev/null
    
    # Compare
    if diff -q /tmp/cpp_$GNUM.txt /tmp/rust_$GNUM.txt > /dev/null 2>&1; then
        echo "✓ Game $GNUM: PASS"
    else
        echo "✗ Game $GNUM: FAIL"
        echo "  Showing diff:"
        diff -u /tmp/cpp_$GNUM.txt /tmp/rust_$GNUM.txt | head -30
        ALL_PASS=false
    fi
done

echo ""
if [ "$ALL_PASS" = true ]; then
    echo "✅ PASS: Outputs match perfectly!"
    exit 0
else
    echo "❌ FAIL: Outputs differ"
    exit 1
fi
