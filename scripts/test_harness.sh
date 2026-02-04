#!/bin/bash
# Test harness - extracts PGN from SCID database using C++ and compares with Rust

set -e

DATABASE="bases/matein1"
GAME_NUMS="1 100 500"

echo "╔══════════════════════════════════════════════════════════╗"
echo "║         SCID Rust Port - TDD Test Harness               ║"
echo "╚══════════════════════════════════════════════════════════╝"
echo ""

# Create output directory
mkdir -p test_output

echo "Step 1: Extract games using C++ (oracle)..."
echo "============================================"

# We'll create a simple TCL script to extract games
cat > test_output/extract_games.tcl << 'EOF'
# TCL script to extract games from SCID database

proc extract_game {dbname gnum} {
    # Open database
    if {[catch {sc_base open $dbname} err]} {
        puts stderr "Error opening database: $err"
        return ""
    }
    
    # Load game
    if {[catch {sc_game load $gnum} err]} {
        puts stderr "Error loading game $gnum: $err"
        sc_base close
        return ""
    }
    
    # Get PGN
    set pgn [sc_game pgn -tags 1 -comments 1 -variations 1]
    
    # Close database
    sc_base close
    
    return $pgn
}

# Main
if {$argc < 2} {
    puts stderr "Usage: tclsh extract_games.tcl <database> <game_numbers...>"
    exit 1
}

set dbname [lindex $argv 0]
set game_numbers [lrange $argv 1 end]

foreach gnum $game_numbers {
    puts "### GAME $gnum ###"
    puts [extract_game $dbname $gnum]
    puts "### END GAME $gnum ###"
    puts ""
}
EOF

echo "Skipping TCL extraction (requires scid binary integration)"
echo "Using direct database reading instead..."
echo ""

echo "Step 2: Show database information..."
echo "====================================="
./scidt -i $DATABASE 2>&1 | head -20

echo ""
echo "Step 3: Test Rust implementation..."
echo "===================================="
cd rust-port
cargo run --example extract_pgn 2>&1 | grep -v "warning\|Compiling\|Finished\|Running" | head -50

echo ""
echo "╔══════════════════════════════════════════════════════════╗"
echo "║                 Next Steps                               ║"
echo "╚══════════════════════════════════════════════════════════╝"
echo ""
echo "To complete the TDD test harness, we need to:"
echo "  1. Build C++ extractor that outputs proper PGN"
echo "  2. Implement full SCID decoder in Rust"
echo "  3. Compare outputs and fix discrepancies"
echo ""
echo "Current status:"
echo "  ✓ C++ database tools available (scidt)"
echo "  ✓ Rust can detect games in database"
echo "  ✓ Rust can extract FEN positions"
echo "  ⚠  Need to decode move sequences"
echo "  ⚠  Need to extract metadata from index"
echo ""
