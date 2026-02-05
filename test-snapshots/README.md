# Test Snapshots

This directory contains PGN snapshots from both the C++ oracle (`tcscid`) and the Rust implementation for comparison.

## Contents

- `tcscid/` - PGN output from SCID's C++ implementation (via tcscid CLI)
- `rust/` - PGN output from Rust port (rust-port/examples/rust_extractor.rs)

## Test Coverage

The snapshots include a variety of game types:

### matein1 (Mate in 1 puzzles)
- Games: 1, 5, 10, 50, 100
- Features: Custom FEN positions, simple tactics

### matein2 (Mate in 2 puzzles)
- Games: 1, 4, 10, 124
- Features: 
  - Game 4: Comments in main line
  - Game 124: Pre-game comments
  - Custom FEN positions

### tactics (Tactical puzzles)
- Games: 1, 16, 86, 100
- Features:
  - Game 16: Variations without comments
  - Game 86: Comments inside variations with NAGs
  - Complex positions

### endings (Endgame positions)
- Games: 1, 10, 33
- Features: Simple endgame positions with custom FENs

## Generating Snapshots

Run `./generate-snapshots.sh` from the repository root to regenerate all snapshots.

## Comparing Outputs

```bash
# Compare specific game
diff test-snapshots/tcscid/matein1-game-1.pgn test-snapshots/rust/matein1-game-1.pgn

# Check all diffs
for f in test-snapshots/tcscid/*.pgn; do 
    basename=$(basename $f)
    diff -u test-snapshots/tcscid/$basename test-snapshots/rust/$basename
done
```

## Current Status

All 16 snapshot pairs should match exactly (character-for-character identical).

If there are differences, it indicates a regression in the Rust implementation.
