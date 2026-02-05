#!/bin/bash
# Generate snapshot files for manual inspection

# Create directories
mkdir -p test-snapshots/tcscid
mkdir -p test-snapshots/rust

# Generate snapshots for a variety of games from different databases

# matein1 - simple tactics
for GNUM in 1 5 10 50 100; do
    echo "Generating matein1 game $GNUM..."
    ./tcscid << EOF > test-snapshots/tcscid/matein1-game-$GNUM.pgn 2>/dev/null
sc_base open bases/matein1
sc_game load $GNUM
puts -nonewline "### GAME $GNUM ###\n"
puts [sc_game pgn]
puts -nonewline "### END GAME $GNUM ###\n\n"
exit
EOF
    cargo run --example rust_extractor --manifest-path rust-port/Cargo.toml bases/matein1 $GNUM > test-snapshots/rust/matein1-game-$GNUM.pgn 2>/dev/null
done

# matein2 - more tactics (includes game 4 with comments)
for GNUM in 1 4 10 124; do
    echo "Generating matein2 game $GNUM..."
    ./tcscid << EOF > test-snapshots/tcscid/matein2-game-$GNUM.pgn 2>/dev/null
sc_base open bases/matein2
sc_game load $GNUM
puts -nonewline "### GAME $GNUM ###\n"
puts [sc_game pgn]
puts -nonewline "### END GAME $GNUM ###\n\n"
exit
EOF
    cargo run --example rust_extractor --manifest-path rust-port/Cargo.toml bases/matein2 $GNUM > test-snapshots/rust/matein2-game-$GNUM.pgn 2>/dev/null
done

# tactics - includes game 86 with comments in variations
for GNUM in 1 16 86 100; do
    echo "Generating tactics game $GNUM..."
    ./tcscid << EOF > test-snapshots/tcscid/tactics-game-$GNUM.pgn 2>/dev/null
sc_base open bases/tactics
sc_game load $GNUM
puts -nonewline "### GAME $GNUM ###\n"
puts [sc_game pgn]
puts -nonewline "### END GAME $GNUM ###\n\n"
exit
EOF
    cargo run --example rust_extractor --manifest-path rust-port/Cargo.toml bases/tactics $GNUM > test-snapshots/rust/tactics-game-$GNUM.pgn 2>/dev/null
done

# endings - simple endgame positions
for GNUM in 1 10 33; do
    echo "Generating endings game $GNUM..."
    ./tcscid << EOF > test-snapshots/tcscid/endings-game-$GNUM.pgn 2>/dev/null
sc_base open bases/endings
sc_game load $GNUM
puts -nonewline "### GAME $GNUM ###\n"
puts [sc_game pgn]
puts -nonewline "### END GAME $GNUM ###\n\n"
exit
EOF
    cargo run --example rust_extractor --manifest-path rust-port/Cargo.toml bases/endings $GNUM > test-snapshots/rust/endings-game-$GNUM.pgn 2>/dev/null
done

echo ""
echo "Snapshots generated in test-snapshots/"
echo ""
echo "Summary:"
echo "========"
ls -1 test-snapshots/tcscid/ | wc -l | xargs echo "tcscid snapshots:"
ls -1 test-snapshots/rust/ | wc -l | xargs echo "rust snapshots:"
echo ""
echo "To compare a specific game:"
echo "  diff test-snapshots/tcscid/matein1-game-1.pgn test-snapshots/rust/matein1-game-1.pgn"
