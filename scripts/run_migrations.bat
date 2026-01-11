@echo off
setlocal enabledelayedexpansion

:: AlphaField Migration Runner
:: Usage: run_migrations.bat [--docker]

:: Check if Python is available
where python >nul 2>nul
if %ERRORLEVEL% neq 0 (
    echo Error: Python is not installed or not in PATH
    echo Please install Python 3.6+ from https://www.python.org/
    exit /b 1
)

:: Check if we should use Docker
set USE_DOCKER=0
for %%a in (%*) do (
    if "%%a"=="--docker" (
        set USE_DOCKER=1
    )
)

:: Change to the project root directory
cd /d "%~dp0.."

:: Check if psycopg2 is installed
python -c "import psycopg2" >nul 2>nul
if %ERRORLEVEL% neq 0 (
    echo Installing required Python packages...
    pip install psycopg2-binary
    if %ERRORLEVEL% neq 0 (
        echo Error: Failed to install psycopg2-binary
        echo Please install it manually: pip install psycopg2-binary
        exit /b 1
    )
)

:: Run the migration script
if %USE_DOCKER%==1 (
    echo Running migrations for Docker database...
    python scripts/run_migrations.py --docker
) else (
    echo Running migrations for local database...
    python scripts/run_migrations.py
)

:: Check exit code
if %ERRORLEVEL% neq 0 (
    echo Migration failed with error code %ERRORLEVEL%
    exit /b %ERRORLEVEL%
)

echo Migrations completed successfully!
exit /b 0