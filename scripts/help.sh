#!/bin/bash
# Quick reference guide for SCID to Rust port

cat << 'EOF'
╔══════════════════════════════════════════════════════════════════════════════╗
║              SCID → Rust Port: Quick Reference Guide                         ║
╚══════════════════════════════════════════════════════════════════════════════╝

📚 DOCUMENTATION
════════════════

   Main Guide:        docs/rust-port-tdd-approach.md
   Summary:           RUST_PORT_SUMMARY.md
   Rust README:       rust-port/README.md

📂 KEY DIRECTORIES
══════════════════

   C++ Source:        src/game.cpp (original implementation)
   Rust Port:         rust-port/ (new implementation)
   Test Generator:    tools/fixture_generator/
   Test Fixtures:     rust-port/tests/fixtures/ (generated)

🛠️  COMMANDS
═══════════

Generate Test Fixtures:
   ./scripts/generate_test_fixtures.sh

Build Rust Project:
   cd rust-port && cargo build

Run All Tests:
   cd rust-port && cargo test

Run Specific Test:
   cd rust-port && cargo test test_manual_simple_game

Run With Output:
   cd rust-port && cargo test -- --nocapture

Check Code Without Building:
   cd rust-port && cargo check

Format Code:
   cd rust-port && cargo fmt

Lint Code:
   cd rust-port && cargo clippy

📝 TYPICAL WORKFLOW
═══════════════════

1. Pick a feature to implement (e.g., move formatting)

2. Generate test fixtures for that feature:
   ./scripts/generate_test_fixtures.sh --type moves

3. Implement the feature in Rust:
   vim rust-port/src/pgn/moves.rs

4. Add unit tests:
   #[cfg(test)]
   mod tests {
       #[test]
       fn test_simple_move() { ... }
   }

5. Run tests to see progress:
   cd rust-port && cargo test

6. Fix issues until tests pass

7. Commit and move to next feature

🎯 TESTING STRATEGY
═══════════════════

Unit Tests:           Test individual functions/modules
                      Located in same file as implementation
                      
Integration Tests:    Test complete PGN conversion
                      Located in rust-port/tests/
                      
Cross-Validation:     Compare Rust output vs C++ baseline
                      Uses JSON fixtures generated from C++

📊 TRACKING PROGRESS
════════════════════

View test results:
   cd rust-port && cargo test 2>&1 | grep "test result"

See which tests failed:
   cd rust-port && cargo test -- --nocapture

Generate coverage report (requires cargo-tarpaulin):
   cd rust-port && cargo tarpaulin --out Html

🐛 DEBUGGING
════════════

Print fixture details:
   cat rust-port/tests/fixtures/simple_games/simple_game_001.json | jq

Compare expected vs actual PGN:
   cd rust-port && cargo test test_fixtures -- --nocapture

Use Rust debugger:
   rust-lldb target/debug/deps/scid_pgn-*

📖 LEARNING RESOURCES
═════════════════════

Rust Book:            https://doc.rust-lang.org/book/
Rust by Example:      https://doc.rust-lang.org/rust-by-example/
PGN Specification:    http://www.saremba.de/chessgml/standards/pgn/
SCID C++ Source:      src/game.cpp, src/game.h

🆘 TROUBLESHOOTING
══════════════════

Q: Rust tests fail but C++ works?
A: Check that fixtures are up-to-date. Regenerate:
   ./scripts/generate_test_fixtures.sh

Q: Fixture generator won't compile?
A: Ensure SCID C++ code is built first:
   cd tools/fixture_generator && make clean && make

Q: Test says "Fixtures directory not found"?
A: Generate fixtures first:
   ./scripts/generate_test_fixtures.sh

Q: How do I add a new test category?
A: 1. Add to fixture generator (tools/fixture_generator/main.cpp)
   2. Update script (scripts/generate_test_fixtures.sh)
   3. Regenerate fixtures

💡 TIPS
═══════

• Start with simple cases before complex ones
• Each feature should have both unit tests and integration tests
• Use cargo watch for continuous testing: cargo watch -x test
• Write tests before implementation (TDD!)
• Keep PRs small and focused on one feature
• Update documentation as you implement features

📧 QUESTIONS?
═════════════

Read the detailed methodology:  docs/rust-port-tdd-approach.md
Check the Rust README:          rust-port/README.md

╚══════════════════════════════════════════════════════════════════════════════╝

Press Ctrl+C to exit this help.
EOF

# Keep it open so user can read
read -p ""
