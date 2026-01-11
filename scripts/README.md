# AlphaField Migration Runner

This script handles database migrations for both Docker and non-Docker environments.

## Usage

### From Makefile

```bash
# Run migrations for local database
make migrate

# Run migrations for Docker database
make migrate-docker

# Reset and run migrations for local database
make reset-db

# Reset and run migrations for Docker database
make reset-db-docker
```

### Direct Usage

```bash
# Run migrations for local database
cd scripts && run_migrations.bat

# Run migrations for Docker database
cd scripts && run_migrations.bat --docker

# Check migration status (local)
cd scripts && python run_migrations.py --status

# Check migration status (Docker)
cd scripts && python run_migrations.py --docker --status
```

## How It Works

1. **Database Connection**: The script automatically detects whether to use Docker or local database based on the `--docker` flag
2. **Migration Discovery**: It scans the `migrations/` directory for `.sql` files
3. **Ordered Execution**: Migrations are executed in alphabetical order (001_*, 002_*, etc.)
4. **Error Handling**: If any migration fails, the entire process stops and reports the error

## Database Configuration

- **Local Database**: Uses `DATABASE_URL` environment variable or defaults to `postgres://alphafield:alphafield_dev@localhost:5432/alphafield`
- **Docker Database**: Uses the same credentials but connects to the Docker container

## Migration Files

Migration files should be placed in the `migrations/` directory and follow the naming convention:
- `001_description.sql`
- `002_description.sql`
- etc.

Each file should contain valid PostgreSQL SQL statements.

## Requirements

- Python 3.6+ (for running the migration script)
- psycopg2-binary package (installed automatically if missing)
- PostgreSQL client libraries

## Troubleshooting

If you encounter connection issues:

1. **Docker**: Ensure the database container is running with `docker-compose up -d timescaledb`
2. **Local**: Ensure PostgreSQL is running and accessible on the configured port
3. **Credentials**: Verify the database credentials in your `.env` file or environment variables