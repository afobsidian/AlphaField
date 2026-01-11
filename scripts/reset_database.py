#!/usr/bin/env python3
"""
AlphaField Database Reset Script
Drops and recreates the database for testing purposes
"""

import os
import sys
import psycopg2
from psycopg2.extensions import ISOLATION_LEVEL_AUTOCOMMIT

def get_database_config(use_docker=False):
    """Get database configuration"""
    if use_docker:
        return {
            'host': 'localhost',
            'port': '5432',
            'user': 'alphafield',
            'password': 'alphafield_dev',
            'dbname': 'alphafield'
        }
    else:
        # Parse DATABASE_URL
        db_url = os.environ.get("DATABASE_URL", 
                              "postgres://alphafield:alphafield_dev@localhost:5432/alphafield")
        
        # Simple parsing - for more robust parsing, use urllib.parse
        parts = db_url.replace("postgres://", "").split("@")
        if len(parts) == 2:
            user_pass = parts[0].split(":")
            host_port_db = parts[1].split("/")
            
            user = user_pass[0]
            password = user_pass[1] if len(user_pass) > 1 else ''
            
            host_port = host_port_db[0].split(":")
            host = host_port[0]
            port = host_port[1] if len(host_port) > 1 else '5432'
            
            dbname = host_port_db[1] if len(host_port_db) > 1 else 'alphafield'
            
            return {
                'host': host,
                'port': port,
                'user': user,
                'password': password,
                'dbname': dbname
            }
        
        return None

def reset_database(use_docker=False):
    """Drop and recreate the database"""
    config = get_database_config(use_docker)
    if not config:
        print("Failed to parse database configuration")
        sys.exit(1)
    
    print(f"Resetting database: {config['dbname']} on {config['host']}:{config['port']}")
    
    try:
        # Connect to postgres database (not the target database)
        conn = psycopg2.connect(
            host=config['host'],
            port=config['port'],
            user=config['user'],
            password=config['password'],
            dbname='postgres'
        )
        conn.set_isolation_level(ISOLATION_LEVEL_AUTOCOMMIT)
        
        # Check if database exists
        with conn.cursor() as cursor:
            cursor.execute("SELECT 1 FROM pg_database WHERE datname = %s", (config['dbname'],))
            db_exists = cursor.fetchone() is not None
        
        if db_exists:
            print(f"Dropping existing database: {config['dbname']}")
            # Terminate all connections to the database
            with conn.cursor() as cursor:
                cursor.execute("""
                    SELECT pg_terminate_backend(pg_stat_activity.pid)
                    FROM pg_stat_activity
                    WHERE pg_stat_activity.datname = %s
                    AND pid <> pg_backend_pid()
                """, (config['dbname'],))
            
            # Drop the database
            with conn.cursor() as cursor:
                cursor.execute(f"DROP DATABASE {config['dbname']}")
        
        # Create the database
        print(f"Creating new database: {config['dbname']}")
        with conn.cursor() as cursor:
            cursor.execute(f"CREATE DATABASE {config['dbname']} OWNER {config['user']}")
        
        conn.close()
        print("Database reset completed successfully!")
        
    except Exception as e:
        print(f"Failed to reset database: {e}")
        sys.exit(1)

def main():
    import argparse
    parser = argparse.ArgumentParser(description="AlphaField Database Reset Script")
    parser.add_argument("--docker", action="store_true", 
                       help="Use Docker database configuration")
    
    args = parser.parse_args()
    reset_database(use_docker=args.docker)

if __name__ == "__main__":
    main()