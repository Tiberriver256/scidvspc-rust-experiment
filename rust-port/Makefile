# Makefile for SCID Rust Port Development

.PHONY: help build test check fmt clippy clean fixtures validate doc coverage watch

# Default target
help:
	@echo "SCID Rust Port - Development Commands"
	@echo ""
	@echo "Building:"
	@echo "  make build          - Build the project"
	@echo "  make build-release  - Build with optimizations"
	@echo "  make check          - Quick compile check (no build)"
	@echo ""
	@echo "Testing:"
	@echo "  make test           - Run all tests"
	@echo "  make test-unit      - Run only unit tests"
	@echo "  make test-integ     - Run integration tests"
	@echo "  make test-verbose   - Run tests with output"
	@echo "  make fixtures       - Generate test fixtures from C++"
	@echo "  make validate       - Cross-validate Rust vs C++"
	@echo ""
	@echo "Code Quality:"
	@echo "  make fmt            - Format code"
	@echo "  make clippy         - Run linter"
	@echo "  make doc            - Generate documentation"
	@echo "  make coverage       - Generate test coverage report"
	@echo ""
	@echo "Development:"
	@echo "  make watch          - Auto-rebuild on file changes"
	@echo "  make clean          - Clean build artifacts"
	@echo "  make status         - Show implementation status"
	@echo ""

# Building
build:
	cargo build

build-release:
	cargo build --release

check:
	cargo check

# Testing
test:
	cargo test

test-unit:
	cargo test --lib

test-integ:
	cargo test --test '*'

test-verbose:
	cargo test -- --nocapture

fixtures:
	@echo "Generating test fixtures from C++..."
	cd .. && ./scripts/generate_test_fixtures.sh

validate:
	@echo "Running cross-validation..."
	cd .. && ./scripts/cross_validate.sh

# Code quality
fmt:
	cargo fmt

clippy:
	cargo clippy -- -D warnings

doc:
	cargo doc --open --no-deps

coverage:
	@echo "Generating coverage report..."
	@which cargo-tarpaulin > /dev/null || cargo install cargo-tarpaulin
	cargo tarpaulin --out Html --output-dir coverage
	@echo "Coverage report: coverage/index.html"

# Development
watch:
	@echo "Watching for changes (install with: cargo install cargo-watch)..."
	@which cargo-watch > /dev/null || cargo install cargo-watch
	cargo watch -x check -x test

clean:
	cargo clean
	rm -rf coverage/
	rm -f ../test-progress.md

status:
	@echo "=== Implementation Status ==="
	@echo ""
	@cargo test 2>&1 | grep "test result"
	@echo ""
	@echo "See test-progress.md for detailed breakdown"
	@if [ -f ../test-progress.md ]; then \
		cat ../test-progress.md | grep -A 20 "Implementation Status"; \
	fi

# Development workflow shortcuts
dev: check test

quick: check
	cargo test --lib

full: clean build test clippy

ci: fmt clippy test

# Help for new contributors
getting-started:
	@echo "=== Getting Started ==="
	@echo ""
	@echo "1. Generate test fixtures:"
	@echo "   make fixtures"
	@echo ""
	@echo "2. Run tests:"
	@echo "   make test"
	@echo ""
	@echo "3. See what needs work:"
	@echo "   make validate"
	@echo ""
	@echo "4. Start developing with auto-rebuild:"
	@echo "   make watch"
	@echo ""
	@echo "5. Before committing:"
	@echo "   make ci"
	@echo ""
	@echo "See README.md for more details."
