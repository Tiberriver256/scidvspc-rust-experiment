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

## Known Issues

### Databases "one" and "five"

The databases from https://github.com/nloding/scidtopgn use a different encoding format that is not yet supported by the Rust implementation. These databases use SCID version 512 format which may have different move encoding than the version 400+ format used by our test databases.

Symptoms:
- Rust outputs no moves (just result)
- C++ (tcscid) reads them correctly
- Move decoder throws "Invalid move" errors

This needs further investigation by comparing the binary move encoding between formats.

## Update: Piece List Order Issue Discovered

Investigation revealed that the "one" and "five" databases fail due to **incorrect piece list ordering** in the Rust implementation.

### Root Cause

SCID's move encoding uses piece list indices. For the standard starting position, C++ builds the piece list in this exact order:
- 0: King (e1)
- 1-7: Rook (a1), Knight (b1), Bishop (c1), Queen (d1), Bishop (f1), Knight (g1), Rook (h1)  
- 8-15: Pawns (a2-h2)

The Rust implementation scans pieces in FEN order (rank 8→1, file a→h), which for White produces:
- Pawns a2-h2 BEFORE rank-1 pieces
- This results in wrong piece indices

When move byte 0xCF (piece #12, which should be pawn e2) is decoded:
- C++ expects piece #12 = pawn e2  
- Rust has piece #12 = pawn a2 (due to FEN scanning + King swap)
- Result: "Invalid move" error

### Solution Needed

The `build_piece_list()` function needs to match C++'s exact ordering algorithm, not FEN scan order. This affects ALL positions, not just standard start.

For now, the Rust implementation works correctly on databases that were tested (matein1-4, tactics, endings) but fails on "one" and "five".
