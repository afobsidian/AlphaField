#!/bin/bash
# AlphaField Migration Runner
# Usage: ./run_migrations.sh [--docker]

set -e  # Exit on error

# Check if Python is available
if ! command -v python3 &> /dev/null; then
    echo "Error: Python is not installed or not in PATH"
    echo "Please install Python 3.6+ using: sudo dnf install python3"
    exit 1
fi

# Check if we should use Docker
USE_DOCKER=0
for arg in "$@"; do
    if [ "$arg" == "--docker" ]; then
        USE_DOCKER=1
    fi
done

# Change to the project root directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR/.."

# Check if psycopg2 is installed
if ! python3 -c "import psycopg2" &> /dev/null; then
    echo "Installing required Python packages..."
    pip3 install psycopg2-binary
    if [ $? -ne 0 ]; then
        echo "Error: Failed to install psycopg2-binary"
        echo "Please install it manually: pip3 install psycopg2-binary"
        exit 1
    fi
fi

# Run the migration script
if [ $USE_DOCKER -eq 1 ]; then
    echo "Running migrations for Docker database..."
    python3 scripts/run_migrations.py --docker
else
    echo "Running migrations for local database..."
    python3 scripts/run_migrations.py
fi

# Check exit code
EXIT_CODE=$?
if [ $EXIT_CODE -ne 0 ]; then
    echo "Migration failed with error code $EXIT_CODE"
    exit $EXIT_CODE
fi

echo "Migrations completed successfully!"
exit 0
