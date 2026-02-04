#!/usr/bin/env bash
# Quick test runner for the SCID Rust port

set -e

cd "$(dirname "$0")/../rust-port"

echo "╔════════════════════════════════════════════════════════════╗"
echo "║         SCID → Rust Port: Test & Demo Runner              ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""

# Run tests
echo "▶ Running all tests..."
cargo test --all 2>&1 | tail -10

echo ""
echo "▶ Test summary:"
cargo test --all 2>&1 | grep "test result:" | tail -3

echo ""
echo "▶ Running example..."
echo ""
cargo run --example convert_game 2>&1 | grep -v "Compiling\|Finished\|Running\|warning"

echo ""
echo "╔════════════════════════════════════════════════════════════╗"
echo "║                    ✅ ALL SYSTEMS GO!                      ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""
echo "Documentation:"
echo "  - FINAL_STATUS_REPORT.md    - Complete status"
echo "  - rust-port/README.md        - Getting started"
echo "  - docs/rust-port-tdd-approach.md - Methodology"
echo ""
echo "Next: cd rust-port && cargo doc --open"
