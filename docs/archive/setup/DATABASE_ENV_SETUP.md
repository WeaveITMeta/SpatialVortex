# Database Environment Setup with .env

**Date**: October 30, 2025  
**Status**: ✅ CONFIGURED  

---

## Quick Start

### 1. Copy Environment Template
```bash
cp .env.example .env
```

### 2. Edit DATABASE_URL
```bash
# Edit .env file
DATABASE_URL=postgresql://username:password@localhost:5432/spatial_vortex
```

### 3. Create Database
```bash
createdb spatial_vortex
```

### 4. Use in Code
```rust
use spatial_vortex::storage::{SpatialDatabase, confidence_lake::PostgresConfidenceLake};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Automatically loads from DATABASE_URL in .env
    let db = SpatialDatabase::from_env().await?;
    let lake = PostgresConfidenceLake::from_env().await?;
    
    Ok(())
}
```

---

## Environment Variable Configuration

### .env File
Create a `.env` file in the project root:

```bash
# Database Configuration
DATABASE_URL=postgresql://username:password@localhost:5432/spatial_vortex

# Optional: Connection pooling
DB_POOL_SIZE=32
DATABASE_TIMEOUT_SECONDS=10
PREPARED_STATEMENTS=true
```

### .env.example
The project includes `.env.example` with all configuration options. Key database settings:

```bash
# Line 130
DATABASE_URL=postgresql://username:password@localhost:5432/spatial_vortex
DATABASE_TIMEOUT_SECONDS=10
PREPARED_STATEMENTS=true
```

---

## Database Connection Methods

### Method 1: From Environment (Recommended)

Both `SpatialDatabase` and `PostgresConfidenceLake` support loading from `.env`:

```rust
use spatial_vortex::storage::SpatialDatabase;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Loads DATABASE_URL from .env file
    let db = SpatialDatabase::from_env().await?;
    
    // Initialize schema
    db.initialize_schema().await?;
    
    Ok(())
}
```

```rust
use spatial_vortex::storage::confidence_lake::PostgresConfidenceLake;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Loads DATABASE_URL from .env file
    let lake = PostgresConfidenceLake::from_env().await?;
    
    // Use lake...
    Ok(())
}
```

### Method 2: Explicit Connection String

For cases where you need explicit control:

```rust
let db = SpatialDatabase::new(
    "postgresql://localhost/spatial_vortex"
).await?;

let lake = PostgresConfidenceLake::new(
    "postgresql://localhost/confidence_lake"
).await?;
```

---

## Connection String Format

### Basic Format
```
postgresql://[user[:password]@][host][:port][/database][?param=value]
```

### Examples

**Local Development**:
```bash
DATABASE_URL=postgresql://localhost/spatial_vortex
```

**With Credentials**:
```bash
DATABASE_URL=postgresql://user:password@localhost:5432/spatial_vortex
```

**Production**:
```bash
DATABASE_URL=postgresql://user:pass@db.example.com:5432/spatialvortex?sslmode=require
```

**Docker/Container**:
```bash
DATABASE_URL=postgresql://postgres:postgres@db:5432/spatial_vortex
```

**With SSL**:
```bash
DATABASE_URL=postgresql://user:pass@host:5432/db?sslmode=require&sslrootcert=/path/to/cert
```

---

## How It Works

### 1. dotenv Loading

Both database classes automatically load `.env` when using `from_env()`:

```rust
pub async fn from_env() -> Result<Self> {
    // Load .env file if present (silent fail if not found)
    let _ = dotenv::dotenv();
    
    // Read DATABASE_URL
    let database_url = std::env::var("DATABASE_URL")
        .context("DATABASE_URL not set")?;
    
    Self::new(&database_url).await
}
```

### 2. Environment Variable Priority

1. **System environment** (highest priority)
2. **.env file** (loaded by dotenv)
3. **Default values** (if any)

### 3. Example Flow

```
Application starts
    ↓
Calls SpatialDatabase::from_env()
    ↓
dotenv::dotenv() loads .env file
    ↓
Reads DATABASE_URL from environment
    ↓
Creates PostgreSQL connection pool
    ↓
Ready to use
```

---

## Database Setup Scripts

### Development Setup

```bash
#!/bin/bash
# setup_dev_db.sh

# Create database
createdb spatial_vortex

# Create .env from template
cp .env.example .env

# Edit .env (manual step)
echo "Please edit .env and set your DATABASE_URL"
```

### Production Setup

```bash
#!/bin/bash
# setup_prod_db.sh

# Read from environment
if [ -z "$DATABASE_URL" ]; then
    echo "ERROR: DATABASE_URL not set"
    exit 1
fi

# Test connection
psql "$DATABASE_URL" -c "SELECT version();"

# Run migrations (handled by application)
echo "Database ready. Run application to initialize schema."
```

---

## Database Initialization

### Automatic Schema Creation

Both databases create their schemas automatically on first connection:

```rust
// SpatialDatabase
let db = SpatialDatabase::from_env().await?;
db.initialize_schema().await?;  // Creates tables & indexes

// PostgresConfidenceLake  
let lake = PostgresConfidenceLake::from_env().await?;
// Schema created automatically in new()
```

### Manual Schema Creation

If needed, you can create schemas manually:

**SpatialDatabase**:
```sql
CREATE TABLE flux_matrices (
    id UUID PRIMARY KEY,
    subject TEXT NOT NULL,
    data JSONB NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_flux_matrices_subject ON flux_matrices(subject);
```

**Confidence Lake**:
```sql
CREATE TABLE flux_matrices (
    id BIGINT PRIMARY KEY,
    confidence DOUBLE PRECISION NOT NULL,
    flux_position INTEGER NOT NULL,
    is_sacred BOOLEAN NOT NULL,
    ethos DOUBLE PRECISION NOT NULL,
    logos DOUBLE PRECISION NOT NULL,
    pathos DOUBLE PRECISION NOT NULL,
    mode TEXT NOT NULL,
    processing_time_ms INTEGER NOT NULL,
    data BYTEA NOT NULL,
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_confidence ON flux_matrices(confidence DESC);
CREATE INDEX idx_sacred ON flux_matrices(is_sacred, flux_position);
```

---

## Configuration Examples

### Development (.env)
```bash
# Local PostgreSQL
DATABASE_URL=postgresql://localhost/spatial_vortex
RUST_LOG=debug
DEVELOPMENT_MODE=true
```

### Testing (.env.test)
```bash
# Separate test database
DATABASE_URL=postgresql://localhost/spatial_vortex_test
```

### Production (Environment Variables)
```bash
# Set in system/container environment
export DATABASE_URL="postgresql://user:pass@prod-db:5432/spatialvortex?sslmode=require"
export DB_POOL_SIZE=64
export DATABASE_TIMEOUT_SECONDS=30
```

### Docker Compose
```yaml
version: '3.8'
services:
  app:
    environment:
      - DATABASE_URL=postgresql://postgres:postgres@db:5432/spatial_vortex
  
  db:
    image: postgres:16
    environment:
      - POSTGRES_DB=spatial_vortex
      - POSTGRES_USER=postgres
      - POSTGRES_PASSWORD=postgres
```

---

## Code Integration Examples

### Complete Application Setup

```rust
use spatial_vortex::storage::{SpatialDatabase, confidence_lake::PostgresConfidenceLake};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env file and initialize databases
    let db = SpatialDatabase::from_env().await?;
    let lake = PostgresConfidenceLake::from_env().await?;
    
    // Initialize schemas
    db.initialize_schema().await?;
    
    // Use databases
    println!("✅ Databases connected!");
    
    Ok(())
}
```

### With Error Handling

```rust
#[tokio::main]
async fn main() -> Result<()> {
    // Try to load from environment
    let db = match SpatialDatabase::from_env().await {
        Ok(db) => db,
        Err(e) => {
            eprintln!("❌ Database connection failed: {}", e);
            eprintln!("💡 Make sure DATABASE_URL is set in .env file");
            return Err(e.into());
        }
    };
    
    println!("✅ Connected to database");
    Ok(())
}
```

### With Fallback

```rust
async fn get_database() -> Result<SpatialDatabase> {
    // Try .env first
    if let Ok(db) = SpatialDatabase::from_env().await {
        return Ok(db);
    }
    
    // Fallback to default
    println!("⚠️  Using default connection");
    SpatialDatabase::new("postgresql://localhost/spatial_vortex").await
}
```

---

## Testing

### Test with .env
```rust
#[tokio::test]
async fn test_database_connection() {
    dotenv::dotenv().ok();
    
    let db = SpatialDatabase::from_env()
        .await
        .expect("Failed to connect");
    
    // Test query
    // ...
}
```

### Test with Override
```rust
#[tokio::test]
async fn test_with_test_db() {
    std::env::set_var(
        "DATABASE_URL",
        "postgresql://localhost/test_db"
    );
    
    let db = SpatialDatabase::from_env().await.unwrap();
    // ...
}
```

---

## Troubleshooting

### Error: "DATABASE_URL not set"
```
Error: DATABASE_URL environment variable not set
```

**Solution**:
1. Create `.env` file: `cp .env.example .env`
2. Edit `.env` and set `DATABASE_URL`
3. Verify: `cat .env | grep DATABASE_URL`

### Error: Connection refused
```
Error: connection to server at "localhost" failed
```

**Solution**:
```bash
# Check PostgreSQL is running
pg_ctl status

# Start if needed
pg_ctl start

# Or via service
sudo systemctl start postgresql
```

### Error: Database does not exist
```
Error: database "spatial_vortex" does not exist
```

**Solution**:
```bash
createdb spatial_vortex
```

### Error: Password authentication failed
```
Error: password authentication failed for user "username"
```

**Solution**: Update credentials in `.env`:
```bash
DATABASE_URL=postgresql://correct_user:correct_pass@localhost/spatial_vortex
```

---

## Security Best Practices

### 1. Never Commit .env
```bash
# .gitignore should include:
.env
.env.local
.env.*.local
```

### 2. Use Strong Credentials
```bash
# Generate secure password
openssl rand -base64 32

# Use in DATABASE_URL
DATABASE_URL=postgresql://user:SECURE_PASSWORD_HERE@host/db
```

### 3. Restrict Permissions
```bash
# Make .env readable only by owner
chmod 600 .env
```

### 4. Production Secrets
Don't use `.env` in production. Use:
- **Kubernetes**: Secrets
- **Docker**: Secrets/Environment
- **Cloud**: Parameter Store, Secrets Manager
- **Bare Metal**: System environment

---

## Summary

✅ **Both databases support `.env`** - Use `from_env()` method  
✅ **DATABASE_URL variable** - Set in `.env` file  
✅ **dotenv integration** - Automatically loads on startup  
✅ **Consistent API** - Same pattern for both databases  
✅ **Error handling** - Clear messages if not configured  

**Recommended Usage**:
```rust
// Always use from_env() in production code
let db = SpatialDatabase::from_env().await?;
let lake = PostgresConfidenceLake::from_env().await?;
```

---

## Related Files

- `.env.example` - Template with all config options
- `src/storage/spatial_database.rs` - SpatialDatabase implementation
- `src/storage/confidence_lake/sqlite_backend.rs` - Lake implementation
- `Cargo.toml` - dotenv dependency
- `POSTGRES_MIGRATION.md` - PostgreSQL migration guide

---

**All databases now properly use DATABASE_URL from .env files!**
