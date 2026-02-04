# TDD Approach for Porting SCID to PGN Conversion to Rust

## Executive Summary

This document outlines a behavioral-focused Test-Driven Development (TDD) approach for porting the SCID database to PGN conversion functionality from C++ to Rust. The strategy emphasizes creating a test harness that validates both implementations against the same behavioral specifications, enabling incremental porting with continuous validation.

## Core Functionality Overview

The main conversion functionality lives in `src/game.cpp` and involves:

1. **Data Structures**:
   - `Game` class: stores game metadata, moves, variations, comments, and NAGs
   - `moveT`: stores individual moves with variations, comments, and annotations
   - `Position`: chess position state
   - Move tree structure with variations using linked lists

2. **Key Functions**:
   - `WriteToPGN()`: Main entry point (calls `WritePGN()`)
   - `WritePGN()`: Generates PGN output with tags, moves, variations, and comments
   - `WriteMoveList()`: Recursive function that writes moves and variations
   - Move encoding/decoding functions for compact binary storage

3. **PGN Features Supported**:
   - Standard seven tags (Event, Site, Date, Round, White, Black, Result)
   - Extended tags (ECO, Elo ratings, EventDate, custom tags)
   - Move variations (nested)
   - Move comments (pre-move and post-move)
   - NAG (Numeric Annotation Glyph) annotations ($1, $2, !, ?, etc.)
   - Multiple output formats (Plain, HTML, LaTeX, Color)
   - FEN for non-standard starting positions

## TDD Strategy

### Phase 1: Test Harness Setup

#### 1.1 Create C++ Test Fixture Generator

**Objective**: Extract test cases from existing C++ code that can be shared with Rust

**Implementation**:
```cpp
// test_fixture_generator.cpp
// Reads SCID databases and outputs test fixtures as JSON

{
  "name": "simple_game",
  "input": {
    "event": "World Championship",
    "site": "Moscow",
    "date": "1985.09.03",
    "round": "1",
    "white": "Kasparov, Garry",
    "black": "Karpov, Anatoly",
    "result": "1-0",
    "moves": [
      {"from": "e2", "to": "e4", "piece": "P", "san": "e4"},
      {"from": "e7", "to": "e5", "piece": "p", "san": "e5"},
      // ... more moves
    ],
    "variations": [],
    "comments": []
  },
  "expected_pgn": "[Event \"World Championship\"]\n..."
}
```

**Test Fixture Categories**:
1. Simple games (no variations, no comments)
2. Games with comments
3. Games with variations (single level)
4. Games with nested variations
5. Games with NAGs
6. Games with non-standard starts (FEN)
7. Games with special characters in tags/comments
8. Edge cases (null moves, underpromotions, etc.)

#### 1.2 Build Test Data Corpus

Extract real test cases from the existing codebase:

```bash
# Script to generate test fixtures
./scripts/generate_test_fixtures.sh
```

This should produce:
- `tests/fixtures/simple_games/*.json` - Basic games
- `tests/fixtures/with_variations/*.json` - Variation handling
- `tests/fixtures/with_comments/*.json` - Comment handling
- `tests/fixtures/special_cases/*.json` - Edge cases
- `tests/fixtures/all_formats/*.json` - HTML, LaTeX, etc.

### Phase 2: Rust Test Infrastructure

#### 2.1 Define Rust Test Framework

```rust
// tests/pgn_conversion_tests.rs
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
struct TestFixture {
    name: String,
    input: GameInput,
    expected_pgn: String,
    options: ConversionOptions,
}

#[derive(Debug, Deserialize, Serialize)]
struct GameInput {
    event: String,
    site: String,
    date: String,
    round: String,
    white: String,
    black: String,
    result: String,
    moves: Vec<MoveInput>,
    variations: Vec<Variation>,
    comments: Vec<Comment>,
    #[serde(skip_serializing_if = "Option::is_none")]
    fen: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ConversionOptions {
    format: PgnFormat,
    include_comments: bool,
    include_variations: bool,
    include_nags: bool,
    short_header: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PgnFormat {
    Plain,
    Html,
    Latex,
    Color,
}

// Test runner that loads fixtures and validates against both implementations
#[test]
fn test_against_cpp_baseline() {
    let fixtures = load_all_fixtures("tests/fixtures");
    
    for fixture in fixtures {
        let rust_output = convert_to_pgn_rust(&fixture.input, &fixture.options);
        
        // Compare against expected output (generated from C++)
        assert_pgn_equivalent(&rust_output, &fixture.expected_pgn, &fixture.name);
    }
}

// Helper to compare PGN output allowing for whitespace differences
fn assert_pgn_equivalent(actual: &str, expected: &str, test_name: &str) {
    let actual_normalized = normalize_pgn(actual);
    let expected_normalized = normalize_pgn(expected);
    
    if actual_normalized != expected_normalized {
        println!("Test failed: {}", test_name);
        println!("Expected:\n{}", expected);
        println!("Actual:\n{}", actual);
        
        // Show diff
        show_diff(&expected_normalized, &actual_normalized);
        panic!("PGN output does not match");
    }
}

fn normalize_pgn(pgn: &str) -> String {
    // Normalize whitespace, line endings, etc.
    // This allows for minor formatting differences
    pgn.lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}
```

#### 2.2 Create Cross-Language Test Validator

Build a tool that can run the same tests against both C++ and Rust:

```rust
// tools/cross_validator/main.rs
// Validates that Rust implementation produces same output as C++

use std::process::Command;
use serde_json;

fn main() {
    let fixtures = load_fixtures("tests/fixtures");
    let mut results = TestResults::new();
    
    for fixture in fixtures {
        // Run C++ version
        let cpp_output = run_cpp_converter(&fixture);
        
        // Run Rust version
        let rust_output = run_rust_converter(&fixture);
        
        // Compare
        let matches = compare_outputs(&cpp_output, &rust_output);
        results.record(&fixture.name, matches);
        
        if !matches {
            println!("MISMATCH: {}", fixture.name);
            println!("C++:\n{}", cpp_output);
            println!("Rust:\n{}", rust_output);
        }
    }
    
    results.print_summary();
}
```

### Phase 3: Incremental Implementation

#### 3.1 Implementation Order (Bottom-Up)

Implement in layers, with tests at each layer:

**Layer 1: Core Data Types**
```rust
// src/types.rs
pub struct Square(u8);
pub struct Piece { /* ... */ }
pub struct Move { /* ... */ }
pub struct Position { /* ... */ }

#[cfg(test)]
mod tests {
    // Test individual types
    #[test]
    fn test_square_creation() { /* ... */ }
    
    #[test]
    fn test_move_validation() { /* ... */ }
}
```

**Layer 2: Game Representation**
```rust
// src/game.rs
pub struct Game {
    event: String,
    site: String,
    white: String,
    black: String,
    moves: Vec<MoveNode>,
    // ...
}

pub struct MoveNode {
    move_data: Move,
    san: String,
    comment: Option<String>,
    nags: Vec<u8>,
    variations: Vec<MoveNode>,
    // ...
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_game_creation() { /* ... */ }
    
    #[test]
    fn test_add_move() { /* ... */ }
    
    #[test]
    fn test_add_variation() { /* ... */ }
}
```

**Layer 3: PGN Tags**
```rust
// src/pgn/tags.rs
pub struct PgnTags { /* ... */ }

impl PgnTags {
    pub fn to_pgn(&self, format: PgnFormat) -> String { /* ... */ }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_standard_tags() {
        let fixture = load_fixture("simple_tags.json");
        let tags = PgnTags::from_fixture(&fixture);
        let pgn = tags.to_pgn(PgnFormat::Plain);
        assert_eq!(pgn, fixture.expected_tags);
    }
    
    #[test]
    fn test_html_tags() {
        // Test HTML format
    }
}
```

**Layer 4: Move Formatting**
```rust
// src/pgn/moves.rs
pub struct MoveFormatter { /* ... */ }

impl MoveFormatter {
    pub fn format_move(&self, move_node: &MoveNode) -> String { /* ... */ }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_simple_move_format() {
        let fixture = load_fixture("simple_moves.json");
        let formatter = MoveFormatter::new(PgnFormat::Plain);
        
        for (move_node, expected) in fixture.moves {
            let actual = formatter.format_move(&move_node);
            assert_eq!(actual, expected);
        }
    }
}
```

**Layer 5: Variation Handling**
```rust
// src/pgn/variations.rs
pub fn format_with_variations(
    moves: &[MoveNode],
    depth: usize,
    options: &PgnOptions,
) -> String {
    // Recursive variation formatting
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_single_variation() {
        let fixture = load_fixture("single_variation.json");
        // Test variation output
    }
    
    #[test]
    fn test_nested_variations() {
        let fixture = load_fixture("nested_variations.json");
        // Test nested variations
    }
}
```

**Layer 6: Complete PGN Writer**
```rust
// src/pgn/writer.rs
pub struct PgnWriter { /* ... */ }

impl PgnWriter {
    pub fn write(&self, game: &Game) -> Result<String, Error> {
        // Complete PGN output
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_complete_games() {
        let fixtures = load_all_fixtures("tests/fixtures/complete_games");
        
        for fixture in fixtures {
            let game = Game::from_fixture(&fixture);
            let writer = PgnWriter::new(fixture.options);
            let pgn = writer.write(&game).unwrap();
            
            assert_pgn_equivalent(&pgn, &fixture.expected_pgn, &fixture.name);
        }
    }
}
```

### Phase 4: Test Categories and Coverage

#### 4.1 Core Test Suites

**Suite 1: Tag Tests**
- Standard 7 tags
- Extended tags (Elo, ECO, etc.)
- Custom tags
- Special characters in tags
- Different formats (Plain, HTML, LaTeX)

**Suite 2: Move Tests**
- Simple moves
- Captures
- Castling (both sides)
- Promotions (queen, rook, bishop, knight)
- En passant
- Null moves

**Suite 3: Annotation Tests**
- Standard NAGs (!, ?, !!, ??, !?, ?!)
- Extended NAGs ($1-$215)
- Comment formatting
- Pre-move vs post-move comments
- Comments with special characters
- Draw markers ([%draw], [%cal], [%csl])

**Suite 4: Variation Tests**
- Single variation
- Multiple variations
- Nested variations (2+ levels)
- Variation comments
- Variation NAGs

**Suite 5: Format Tests**
- Plain PGN
- HTML format
- LaTeX format (with board diagrams)
- Color format

**Suite 6: Edge Cases**
- Empty game
- Game with only tags
- Non-standard starting position (FEN)
- Very long games (200+ moves)
- Very deep variations (10+ levels)
- Unicode in comments/tags
- Games with errors/incomplete data

#### 4.2 Regression Test Suite

As bugs are found and fixed, add them to a regression suite:

```rust
#[test]
fn test_regression_001_variation_comment_spacing() {
    // Bug: Extra space before variation comment
    // Fixed: 2024-02-04
    let fixture = load_fixture("regression/001.json");
    // ...
}
```

### Phase 5: Progress Tracking

#### 5.1 Test Dashboard

Create a dashboard to track implementation progress:

```bash
# Run test suite and generate report
cargo test --test pgn_tests -- --test-threads=1 --format=json | \
    ./tools/generate_report.sh > test_report.html
```

Dashboard shows:
- Total tests: 250
- Passing: 187 (74.8%)
- Failing: 63 (25.2%)
- Coverage by category:
  - Tags: 100% ✓
  - Simple moves: 100% ✓
  - Variations: 65% (in progress)
  - Annotations: 45% (in progress)
  - Formats: 30% (todo)

#### 5.2 Continuous Integration

Set up CI to run tests on every commit:

```yaml
# .github/workflows/rust-port-tests.yml
name: Rust Port Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      
      - name: Generate test fixtures from C++
        run: |
          make test-fixtures
      
      - name: Run Rust tests
        run: |
          cargo test --all-features
      
      - name: Run cross-validation
        run: |
          cargo run --bin cross-validator
      
      - name: Generate coverage report
        run: |
          cargo tarpaulin --out Html --output-dir coverage
      
      - name: Upload results
        uses: actions/upload-artifact@v2
        with:
          name: test-results
          path: |
            coverage/
            test-report.html
```

## Recommended Directory Structure

```
scidvspc-code/
├── src/                          # Existing C++ code
│   ├── game.cpp
│   ├── game.h
│   └── ...
├── rust-port/                    # New Rust implementation
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── types.rs              # Core types
│   │   ├── game.rs               # Game struct
│   │   ├── position.rs           # Position logic
│   │   └── pgn/
│   │       ├── mod.rs
│   │       ├── tags.rs           # PGN tag handling
│   │       ├── moves.rs          # Move formatting
│   │       ├── variations.rs     # Variation handling
│   │       └── writer.rs         # Main PGN writer
│   ├── tests/
│   │   ├── fixtures/             # Test data (JSON)
│   │   │   ├── simple_games/
│   │   │   ├── with_variations/
│   │   │   ├── with_comments/
│   │   │   ├── special_cases/
│   │   │   └── regression/
│   │   ├── pgn_tests.rs          # Main test suite
│   │   └── integration_tests.rs
│   └── tools/
│       ├── fixture_generator/    # C++ tool to generate fixtures
│       │   ├── main.cpp
│       │   └── Makefile
│       └── cross_validator/      # Validates both implementations
│           ├── Cargo.toml
│           └── src/
│               └── main.rs
├── scripts/
│   ├── generate_test_fixtures.sh
│   ├── run_cross_validation.sh
│   └── generate_report.sh
└── docs/
    └── rust-port-tdd-approach.md # This document
```

## Implementation Workflow

### Step-by-Step Process

1. **Generate Test Fixtures** (Week 1)
   ```bash
   cd tools/fixture_generator
   make
   ./fixture_generator --output ../../rust-port/tests/fixtures/
   ```

2. **Validate Fixtures** (Week 1)
   - Verify fixtures cover all edge cases
   - Ensure C++ baseline produces expected output
   - Document any ambiguities

3. **Implement Core Types** (Week 2)
   ```bash
   cd rust-port
   cargo test types::tests --lib
   # Fix until all pass
   ```

4. **Implement Tags Layer** (Week 3)
   ```bash
   cargo test pgn::tags::tests --lib
   # Fix until all pass
   ```

5. **Implement Move Formatting** (Week 4)
   ```bash
   cargo test pgn::moves::tests --lib
   # Fix until all pass
   ```

6. **Implement Variations** (Week 5-6)
   ```bash
   cargo test pgn::variations::tests --lib
   # More complex, may take longer
   ```

7. **Integrate and Test** (Week 7)
   ```bash
   cargo test --all
   cargo run --bin cross-validator
   ```

8. **Optimize and Refine** (Week 8)
   - Profile performance
   - Refactor for clarity
   - Add documentation

## Benefits of This Approach

1. **Safety**: Always have working tests validating behavior
2. **Incremental**: Can port piece by piece
3. **Measurable**: Clear metrics on progress
4. **Confidence**: Tests prove Rust matches C++ behavior
5. **Documentation**: Tests serve as specification
6. **Regression Prevention**: Catch regressions immediately
7. **Cross-Language**: Same tests validate both implementations
8. **Reproducible**: Anyone can verify the port

## Example Test Fixture

```json
{
  "name": "kasparov_karpov_1985_game1",
  "description": "First game of 1985 World Championship match",
  "input": {
    "event": "World Championship",
    "site": "Moscow",
    "date": "1985.09.03",
    "round": "1",
    "white": "Kasparov, Garry",
    "black": "Karpov, Anatoly",
    "white_elo": 2700,
    "black_elo": 2720,
    "result": "1-0",
    "eco": "B44",
    "moves": [
      {
        "number": 1,
        "white": {"san": "e4", "from": "e2", "to": "e4"},
        "black": {"san": "c5", "from": "c7", "to": "c5"}
      },
      {
        "number": 2,
        "white": {"san": "Nf3", "from": "g1", "to": "f3"},
        "black": {"san": "e6", "from": "e7", "to": "e6"}
      }
    ],
    "comments": [
      {
        "after_move": {"number": 1, "color": "white"},
        "text": "The Sicilian Defense"
      }
    ],
    "variations": [],
    "nags": []
  },
  "expected_pgn": "[Event \"World Championship\"]\n[Site \"Moscow\"]\n[Date \"1985.09.03\"]\n[Round \"1\"]\n[White \"Kasparov, Garry\"]\n[Black \"Karpov, Anatoly\"]\n[Result \"1-0\"]\n[WhiteElo \"2700\"]\n[BlackElo \"2720\"]\n[ECO \"B44\"]\n\n1. e4 {The Sicilian Defense} c5 2. Nf3 e6 1-0\n",
  "options": {
    "format": "plain",
    "include_comments": true,
    "include_variations": true,
    "include_nags": true,
    "short_header": false
  }
}
```

## Conclusion

This TDD approach provides a robust, measurable path to porting SCID's PGN conversion from C++ to Rust. By creating a comprehensive test harness upfront and implementing incrementally, you can:

- Ensure behavioral equivalence between implementations
- Track progress objectively
- Catch regressions early
- Build confidence in the Rust implementation
- Create excellent documentation through tests

The key is starting with extensive test fixture generation from the working C++ code, then using those fixtures as the specification for the Rust implementation.
