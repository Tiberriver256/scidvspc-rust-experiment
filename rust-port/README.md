# SCID PGN Converter - Rust Port

This is a Rust port of the SCID database to PGN conversion functionality. The goal is to achieve behavioral equivalence with the C++ implementation while leveraging Rust's safety and modern language features.

## Project Status

🚧 **Work in Progress** 🚧

This is an incremental port following a Test-Driven Development (TDD) approach. Tests are generated from the working C++ implementation to ensure behavioral equivalence.

### Current Implementation Status

- ✅ Core types (Square, Piece, Move, etc.)
- ✅ Game representation
- ✅ Position handling (basic)
- ✅ PGN tag writing
- ⚠️ Move formatting (in progress)
- ❌ Variation handling (todo)
- ❌ Comment formatting (todo)
- ❌ NAG annotations (todo)
- ❌ Multiple output formats (HTML, LaTeX) (todo)

## Getting Started

### Prerequisites

- Rust 1.70+ (install from [rustup.rs](https://rustup.rs/))
- C++ compiler (for generating test fixtures)

### Building

```bash
cd rust-port
cargo build
```

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_manual_simple_game

# Run with output
cargo test -- --nocapture
```

### Generating Test Fixtures

Before running integration tests, you need to generate test fixtures from the C++ implementation:

```bash
cd ..  # Go back to project root
./scripts/generate_test_fixtures.sh
```

This will:
1. Build the fixture generator from C++ code
2. Generate JSON test fixtures in `tests/fixtures/`
3. Validate the fixtures

Once fixtures are generated, run the integration tests:

```bash
cd rust-port
cargo test test_fixtures
```

## Architecture

### Module Organization

```
src/
├── lib.rs              # Library root
├── types.rs            # Core chess types (Square, Piece, Move, etc.)
├── game.rs             # Game representation with moves and metadata
├── position.rs         # Chess position and move generation
└── pgn/
    ├── mod.rs          # PGN module root
    ├── tags.rs         # PGN tag handling
    ├── moves.rs        # Move formatting (TODO)
    ├── variations.rs   # Variation handling (TODO)
    └── writer.rs       # Main PGN writer
```

### Test Structure

```
tests/
├── fixtures/           # Generated test data from C++
│   ├── simple_games/
│   ├── with_variations/
│   ├── with_comments/
│   └── ...
└── pgn_tests.rs        # Integration tests
```

## Development Workflow

### 1. Implement a Feature

```rust
// src/pgn/moves.rs
pub struct MoveFormatter {
    // Implementation
}
```

### 2. Write Unit Tests

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_simple_move() {
        // Test
    }
}
```

### 3. Validate Against C++

```bash
# Generate fixtures for this feature
./scripts/generate_test_fixtures.sh --type moves

# Run tests
cargo test
```

### 4. Measure Progress

```bash
# Generate coverage report
cargo tarpaulin --out Html

# View report
open tarpaulin-report.html
```

## Test-Driven Development Approach

This port follows a behavioral TDD approach where:

1. **Test fixtures are generated from the working C++ implementation**
   - Each fixture contains input data and expected PGN output
   - Fixtures cover all features and edge cases

2. **Rust implementation is validated against these fixtures**
   - Same input → same output as C++
   - Tests are language-agnostic (JSON format)

3. **Progress is measurable**
   - Number of passing tests shows completion percentage
   - Easy to identify what still needs implementation

See [docs/rust-port-tdd-approach.md](../docs/rust-port-tdd-approach.md) for detailed methodology.

## Example Usage

```rust
use scid_pgn::{Game, PgnWriter};
use scid_pgn::types::GameResult;

// Create a game
let mut game = Game::new();
game.event = "World Championship".to_string();
game.white = "Kasparov, Garry".to_string();
game.black = "Karpov, Anatoly".to_string();
game.result = GameResult::White;
game.white_elo = Some(2700);
game.black_elo = Some(2720);

// Add moves (TODO: implement move API)

// Convert to PGN
let writer = PgnWriter::default();
let pgn = writer.write(&game).unwrap();
println!("{}", pgn);
```

## Contributing

When adding new features:

1. Generate test fixtures from C++ for the feature
2. Write Rust implementation
3. Validate against fixtures
4. Add unit tests for Rust-specific logic
5. Update this README with progress

## Testing Philosophy

- **Behavioral equivalence**: Output must match C++ exactly
- **Comprehensive coverage**: Test all edge cases
- **Cross-validation**: Same tests run on both implementations
- **Regression prevention**: Failed tests become permanent fixtures

## Known Differences from C++

This section documents intentional differences:

- **Memory management**: Uses Rust's ownership system instead of manual allocation
- **Error handling**: Uses `Result<T, E>` instead of error codes
- **String handling**: UTF-8 strings throughout (C++ uses char*)
- **Data structures**: Rust-idiomatic (Vec instead of linked lists where appropriate)

## Performance

Performance optimization comes after correctness is established. Current focus is on:

1. ✅ Correctness (matching C++ behavior)
2. ⚠️ Clarity (readable, maintainable code)
3. ❌ Performance (not yet optimized)

## Resources

- [TDD Approach Documentation](../docs/rust-port-tdd-approach.md)
- [Original C++ Implementation](../src/game.cpp)
- [PGN Specification](http://www.saremba.de/chessgml/standards/pgn/pgn-complete.htm)

## License

Same as the parent SCID project.
