# Piece List Order Issue - Investigation Notes

## Summary

The "one" and "five" databases from nloding/scidtopgn fail to extract due to **incorrect piece list ordering** in the Rust move decoder. All databases use SCID v400 format, so this is NOT a version issue - it's an implementation bug.

## Problem

SCID's binary move encoding uses piece list indices (high 4 bits of each move byte). The piece list order MUST match exactly between encoding and decoding.

### C++ Piece List Order (Standard Position)

```
White pieces (from Position::StdStart):
 0: King (e1)
 1: Rook (a1)
 2: Knight (b1)
 3: Bishop (c1)
 4: Queen (d1)
 5: Bishop (f1)
 6: Knight (g1)
 7: Rook (h1)
 8-15: Pawns (a2-h2)
```

### Rust Piece List Order (Current Implementation)

Scans in FEN order (rank 8→1, file a→h):

```
White pieces:
 0: King (e1)      ← Correct (King swap happens)
 1: Pawn (b2)      ← WRONG! Should be Rook a1
 2: Pawn (c2)
 ...
 8: Rook (a1)      ← Should be at position 1
 9: Knight (b1)    ← Should be at position 2
 ...
12: Pawn (a2)      ← WRONG! Should be Pawn e2
```

## Example Failure

Game "one" #1, first move should be `1.e4`:
- Move byte: `0xCF` = piece #12, move type 0xF
- C++ decodes: piece #12 = pawn e2 → e4 ✓
- Rust decodes: piece #12 = pawn a2 → Invalid move! ✗

## Root Cause

### C++ Algorithm

1. **StdStart() explicitly sets piece list**:
   ```cpp
   List[WHITE][0] = E1;  // King
   List[WHITE][1] = A1;  // Rook
   // ... explicit positions
   ```

2. **FEN loading uses AddPiece()**:
   - Scans FEN rank 8→1, file a→h
   - When King encountered: `List[Count] = List[0]; List[0] = King`
   - But for standard position, FEN scan encounters PAWNS FIRST!
   - So List[0] = a2 initially, then King swaps with it

### Rust Algorithm (Current - WRONG)

1. Scans FEN order: rank 8→1, file a→h
2. For White: a8→h8 (empty), ..., **a2→h2 (pawns!)**, a1→h1 (pieces)
3. King swap happens when e1 encountered
4. Result: Pawns before rank-1 pieces

## Why Old Databases Work

Our test databases (matein1-4, tactics, endings) all have **custom starting positions** (SetUp="1" with FEN). For these:
- The piece list is built from the FEN, not standard start
- Apparently the FEN scanning + King swap happens to produce the right order for those specific positions
- OR those games have fewer moves/simpler positions that don't expose the bug

## Solution

Need to understand C++'s exact piece list building algorithm:

1. **For standard position**: Use explicit order like StdStart()
2. **For FEN positions**: Match AddPiece() logic exactly

The challenge is that C++ has TWO code paths:
- `StdStart()`: Explicitly hardcoded list
- `ReadFromFEN() → AddPiece()`: Dynamic list building

We need to figure out which one SCID actually uses when loading games, or match the AddPiece logic perfectly.

## Next Steps

1. Add test for piece list order against C++ output
2. Fix `build_piece_list()` to match C++ exactly
3. Test against "one" database
4. Verify all other databases still work

## Files to Review

- `src/position.cpp`: `StdStart()`, `AddPiece()`, `ReadFromFEN()`
- `rust-port/examples/move_decoder.rs`: `build_piece_list()`

## Test Command

```bash
# Should output moves, not "Invalid move" error:
cargo run --example rust_extractor --manifest-path rust-port/Cargo.toml bases/one 1
```

Expected: 19 moves starting with `1.e4`  
Actual: No moves, "Failed to apply move: Invalid move"
