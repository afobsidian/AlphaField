# AlphaField Makefile

.PHONY: all build test clean fmt lint run-demo run-backtest reset help

# Default target
all: build

# --- Development ---

## Build the project
build:
	cargo build

## Run tests
test:
	cargo test

## Format code
fmt:
	cargo fmt

## Lint code
lint:
	cargo clippy -- -D warnings

# --- Execution ---

## Run the data demo
run-demo:
	cargo run --bin data-demo --release

## Run the Golden Cross backtest example
run-backtest:
	cargo run --example golden_cross_backtest -p alphafield_backtest

# --- Maintenance ---

## Clean build artifacts
clean:
	cargo clean

## Reset the project (clean and re-build)
reset: clean build

## Show help
help:
	@echo "AlphaField Makefile Targets:"
	@echo "  build         - Build the project"
	@echo "  test          - Run tests"
	@echo "  fmt           - Format code"
	@echo "  lint          - Lint code"
	@echo "  run-demo      - Run the data demo"
	@echo "  run-backtest  - Run the Golden Cross backtest"
	@echo "  clean         - Clean build artifacts"
	@echo "  reset         - Clean and re-build"
