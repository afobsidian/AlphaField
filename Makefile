# AlphaField Makefile

.PHONY: all build build-release test clean fmt lint ci run-demo run-backtest run-dashboard docker-build docker-up docker-down reset help

# Default target
all: build

# --- DevOps ---

## Run local CI (fmt, lint, test)
ci: fmt lint test

## Build Docker image
docker-build:
	docker build -t alphafield:latest .

## Start Docker database
docker-db-up:
	docker compose up -d timescaledb

## Stop Docker database
docker-db-down:
	docker compose down -v timescaledb

## Start Docker environment
docker-up:
	docker compose up -d

## Stop Docker environment
docker-down:
	docker compose down

docker-reset:
	docker compose down -v

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

## Lint code (matches CI)
lint:
	cargo clippy --workspace --all-targets -- -D warnings

# --- Execution ---

## Run the data demo
run-demo:
	cargo run --bin data-demo --release

## Run the Golden Cross backtest example
run-backtest:
	cargo run --example golden_cross_backtest -p alphafield-backtest --release

## Run the dashboard server
run-dashboard:
	cargo run --bin dashboard_server --release

# --- Database ---

## Run database migrations (local)
migrate:
	cd scripts && run_migrations.bat

## Run database migrations (Docker)
migrate-docker:
	cd scripts && run_migrations.bat --docker

## Check migration status (local)
migrate-status:
	cd scripts && python run_migrations.py --status

## Check migration status (Docker)
migrate-status-docker:
	cd scripts && python run_migrations.py --docker --status

## Reset database and run migrations
reset-db:
	cd scripts && cargo clean && run_migrations.bat

## Reset database and run migrations (Docker)
reset-db-docker:
	cd scripts && cargo clean && run_migrations.bat --docker

# --- Maintenance ---

## Clean build artifacts
clean:
	cargo clean

## Reset the project (clean and re-build)
reset: clean build

## Show help
help:
	@echo "AlphaField Makefile Targets:"
	@echo ""
	@echo "Development:"
	@echo "  build          - Build the project (debug)"
	@echo "  test           - Run tests"
	@echo "  fmt            - Format code"
	@echo "  lint           - Lint code with Clippy"
	@echo "  ci             - Run local CI (fmt + lint + test)"
	@echo "  clean          - Clean build artifacts"
	@echo "  reset          - Clean and re-build"
	@echo ""
	@echo "Execution:"
	@echo "  run-demo       - Run the data demo"
	@echo "  run-backtest   - Run the Golden Cross backtest"
	@echo "  run-dashboard  - Run the dashboard server"
	@echo ""
	@echo "Database:"
	@echo "  migrate            - Run database migrations (local)"
	@echo "  migrate-docker     - Run database migrations (Docker)"
	@echo "  migrate-status     - Check migration status (local)"
	@echo "  migrate-status-docker - Check migration status (Docker)"
	@echo "  reset-db           - Reset database and run migrations"
	@echo "  reset-db-docker    - Reset database and run migrations (Docker)"
	@echo ""
	@echo "Docker:"
	@echo "  docker-build   - Build Docker image"
	@echo "  docker-db-up   - Start Docker database"
	@echo "  docker-db-down - Stop Docker database"
	@echo "  docker-up      - Start Docker environment"
	@echo "  docker-down    - Stop Docker environment"
	@echo "  docker-reset   - Reset Docker environment"
