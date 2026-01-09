// Migration runner for AlphaField
// Handles both Docker and non-Docker database environments

use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::env;
use std::fs;
use std::path::Path;
use std::process;

#[tokio::main]
async fn main() {
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    
    // Check if we should use Docker database
    let use_docker = args.contains(&"--docker".to_string());
    
    // Set up database URL based on environment
    let database_url = if use_docker {
        // Docker database URL
        "postgres://alphafield:alphafield_dev@localhost:5432/alphafield".to_string()
    } else {
        // Try to get from environment variable, fall back to default
        env::var("DATABASE_URL").unwrap_or_else(|_| 
            "postgres://alphafield:alphafield_dev@localhost:5432/alphafield".to_string()
        )
    };
    
    println!("Running migrations with database URL: {}", database_url);
    
    // Create database connection pool
    let pool = match PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await {
        Ok(pool) => pool,
        Err(e) => {
            eprintln!("Failed to connect to database: {}", e);
            process::exit(1);
        }
    };
    
    println!("Connected to database successfully");
    
    // Run migrations
    let migrations_dir = "migrations";
    let migrations = match list_migrations(migrations_dir) {
        Ok(migrations) => migrations,
        Err(e) => {
            eprintln!("Failed to list migrations: {}", e);
            process::exit(1);
        }
    };
    
    println!("Found {} migrations to run", migrations.len());
    
    // Apply migrations
    for migration in migrations {
        if let Err(e) = apply_migration(&pool, &migration).await {
            eprintln!("Failed to apply migration {}: {}", migration, e);
            process::exit(1);
        }
        println!("✓ Applied migration: {}", migration);
    }
    
    println!("All migrations completed successfully!");
}

/// List all migration files in the migrations directory
fn list_migrations(migrations_dir: &str) -> std::io::Result<Vec<String>> {
    let mut migrations = Vec::new();
    
    let entries = fs::read_dir(migrations_dir)?;
    
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        
        if path.extension().and_then(|s| s.to_str()) == Some("sql") {
            if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
                migrations.push(file_name.to_string());
            }
        }
    }
    
    // Sort migrations by filename (lexicographical order)
    migrations.sort();
    
    Ok(migrations)
}

/// Apply a single migration file
async fn apply_migration(pool: &Pool<Postgres>, migration_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    let migration_path = Path::new("migrations").join(migration_file);
    let sql = fs::read_to_string(&migration_path)?;
    
    // Execute the SQL migration
    sqlx::query(&sql)
        .execute(pool)
        .await?;
    
    Ok(())
}