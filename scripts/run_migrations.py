#!/usr/bin/env python3
"""
AlphaField Migration Runner
Handles database migrations for both Docker and non-Docker environments
"""

import os
import sys
import subprocess
import argparse
from pathlib import Path
import psycopg2
from psycopg2 import sql
from psycopg2.extensions import ISOLATION_LEVEL_AUTOCOMMIT

def get_database_url(use_docker=False):
    """Get the appropriate database URL based on environment"""
    if use_docker:
        # Docker database URL
        return "postgres://alphafield:alphafield_dev@localhost:5432/alphafield"
    else:
        # Try to get from environment variable, fall back to default
        return os.environ.get("DATABASE_URL", 
                           "postgres://alphafield:alphafield_dev@localhost:5432/alphafield")

def test_database_connection(db_url):
    """Test if we can connect to the database"""
    try:
        conn = psycopg2.connect(db_url)
        conn.close()
        return True
    except Exception as e:
        print(f"Failed to connect to database: {e}")
        return False

def get_migration_files(migrations_dir="migrations"):
    """Get all migration files sorted by name"""
    migrations_path = Path(migrations_dir)
    if not migrations_path.exists():
        print(f"Migrations directory not found: {migrations_path}")
        sys.exit(1)
    
    migration_files = []
    for file in migrations_path.glob("*.sql"):
        migration_files.append(file.name)
    
    # Sort migrations by filename (lexicographical order)
    migration_files.sort()
    
    return migration_files

def apply_migration(conn, migration_file, migrations_dir="migrations"):
    """Apply a single migration file"""
    migration_path = Path(migrations_dir) / migration_file
    
    try:
        with open(migration_path, 'r') as f:
            sql_content = f.read()
        
        # Execute the SQL migration
        with conn.cursor() as cursor:
            try:
                # Try to execute the entire SQL content as a single statement first
                cursor.execute(sql_content)
            except Exception as e:
                # If that fails, try to handle "already exists" errors gracefully
                error_msg = str(e)
                if "already exists" in error_msg.lower():
                    print(f"  Skipping (already exists): {migration_file}")
                    return True
                else:
                    raise e
        
        conn.commit()
        return True
    except Exception as e:
        conn.rollback()
        print(f"Failed to apply migration {migration_file}: {e}")
        return False

def create_migrations_table(conn):
    """Create the migrations table if it doesn't exist"""
    try:
        with conn.cursor() as cursor:
            cursor.execute("""
                CREATE TABLE IF NOT EXISTS migrations (
                    id SERIAL PRIMARY KEY,
                    name VARCHAR(255) UNIQUE NOT NULL,
                    applied_at TIMESTAMPTZ DEFAULT NOW()
                )
            """)
        conn.commit()
        return True
    except Exception as e:
        print(f"Failed to create migrations table: {e}")
        return False

def get_applied_migrations(conn):
    """Get list of already applied migrations"""
    try:
        with conn.cursor() as cursor:
            cursor.execute("SELECT name FROM migrations ORDER BY applied_at")
            return [row[0] for row in cursor.fetchall()]
    except Exception as e:
        print(f"Failed to get applied migrations: {e}")
        return []

def mark_migration_applied(conn, migration_file):
    """Mark a migration as applied in the database"""
    try:
        with conn.cursor() as cursor:
            cursor.execute(
                "INSERT INTO migrations (name) VALUES (%s) ON CONFLICT (name) DO NOTHING",
                (migration_file,)
            )
        conn.commit()
        return True
    except Exception as e:
        print(f"Failed to mark migration as applied: {e}")
        return False

def run_migrations(use_docker=False):
    """Run all database migrations"""
    db_url = get_database_url(use_docker)
    print(f"Running migrations with database URL: {db_url}")
    
    if not test_database_connection(db_url):
        sys.exit(1)
    
    print("Connected to database successfully")
    
    # Get migration files
    migration_files = get_migration_files()
    print(f"Found {len(migration_files)} migration files")
    
    # Connect to database
    try:
        conn = psycopg2.connect(db_url)
        conn.set_isolation_level(ISOLATION_LEVEL_AUTOCOMMIT)
        
        # Create migrations table if it doesn't exist
        if not create_migrations_table(conn):
            print("Failed to initialize migrations table")
            sys.exit(1)
        
        # Get already applied migrations
        applied_migrations = get_applied_migrations(conn)
        print(f"Found {len(applied_migrations)} already applied migrations")
        
        # Determine which migrations need to be applied
        pending_migrations = [m for m in migration_files if m not in applied_migrations]
        
        if not pending_migrations:
            print("No pending migrations to apply")
            conn.close()
            return
        
        print(f"Applying {len(pending_migrations)} pending migrations:")
        
        # Apply pending migrations
        for migration_file in pending_migrations:
            print(f"Applying migration: {migration_file}")
            if not apply_migration(conn, migration_file):
                print("Migration failed!")
                sys.exit(1)
            
            if not mark_migration_applied(conn, migration_file):
                print("Failed to mark migration as applied!")
                sys.exit(1)
            
            print(f"[OK] Applied migration: {migration_file}")
        
        conn.close()
        print(f"All {len(pending_migrations)} migrations completed successfully!")
        
    except Exception as e:
        print(f"Database error: {e}")
        sys.exit(1)

def main():
    parser = argparse.ArgumentParser(description="AlphaField Migration Runner")
    parser.add_argument("--docker", action="store_true", 
                       help="Use Docker database configuration")
    
    args = parser.parse_args()
    
    # Check if psycopg2 is available
    try:
        import psycopg2
    except ImportError:
        print("Error: psycopg2 is not installed.")
        print("Please install it with: pip install psycopg2-binary")
        sys.exit(1)
    
    run_migrations(use_docker=args.docker)

if __name__ == "__main__":
    main()