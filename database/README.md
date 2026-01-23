# SpatialVortex Database Setup

## Requirements

- PostgreSQL 12+ (for JSONB support)
- Database connection string

## Quick Start

### 1. Install PostgreSQL

**Windows (via Chocolatey)**:
```powershell
choco install postgresql
```

**macOS (via Homebrew)**:
```bash
brew install postgresql
brew services start postgresql
```

**Linux (Ubuntu/Debian)**:
```bash
sudo apt-get install postgresql postgresql-contrib
sudo systemctl start postgresql
```

### 2. Create Database

```bash
# Connect to PostgreSQL
psql -U postgres

# Create database
CREATE DATABASE spatialvortex;

# Exit
\q
```

### 3. Run Schema Migration

```bash
psql -U postgres -d spatialvortex -f database/schema.sql
```

### 4. Configure Connection String

Set your database URL as an environment variable:

```bash
# Example format
export DATABASE_URL="host=localhost user=postgres password=yourpassword dbname=spatialvortex"
```

## Connection String Format

```
host=<hostname> port=<port> user=<username> password=<password> dbname=<database>
```

**Example**:
```
host=localhost port=5432 user=postgres password=secret dbname=spatialvortex
```

## Database Schema

### Tables

1. **flux_matrices**
   - Stores FluxMatrix data as JSONB
   - Indexed by subject for fast lookups
   - Automatic timestamp management

2. **inference_log**
   - Records inference operations
   - Links to flux_matrices via foreign key
   - Tracks performance metrics

### Views

1. **matrix_statistics**
   - Total matrices count
   - Unique subjects count
   - Average matrix size
   - Creation timestamps

2. **inference_statistics**
   - Total inferences (24h)
   - Average confidence
   - Processing time metrics

## Usage Example

```rust
use spatial_vortex::spatial_database::SpatialDatabase;
use spatial_vortex::models::FluxMatrix;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to database
    let db = SpatialDatabase::new("host=localhost user=postgres dbname=spatialvortex").await?;
    
    // Initialize schema
    db.initialize_schema().await?;
    
    // Create a flux matrix
    let matrix = FluxMatrix::new("physics".to_string());
    
    // Store it
    db.store_matrix(&matrix).await?;
    
    // Retrieve by subject
    let retrieved = db.get_matrix_by_subject("physics").await?;
    
    // Get statistics
    let stats = db.get_statistics().await?;
    println!("Total matrices: {}", stats.total_matrices);
    
    Ok(())
}
```

## Production Deployment

### Connection Pooling

The database layer uses **deadpool-postgres** for efficient connection pooling:
- Default: 16 connections per pool
- Auto-reconnect on connection failure
- Fast recycling method for optimal performance

### Best Practices

1. **Use environment variables** for connection strings (never hardcode)
2. **Enable SSL/TLS** in production:
   ```
   host=db.example.com sslmode=require
   ```
3. **Set up backups** using pg_dump:
   ```bash
   pg_dump -U postgres spatialvortex > backup.sql
   ```
4. **Monitor performance** using `inference_statistics` view
5. **Index optimization**: Schema includes optimal indexes for common queries

### Performance Tuning

**PostgreSQL Configuration** (postgresql.conf):
```ini
# Memory
shared_buffers = 256MB
effective_cache_size = 1GB
work_mem = 16MB

# Connections
max_connections = 100

# JSON
# Already optimized via GIN index on JSONB columns
```

## Maintenance

### Backup

```bash
# Full backup
pg_dump -U postgres spatialvortex > spatialvortex_backup.sql

# Restore
psql -U postgres -d spatialvortex < spatialvortex_backup.sql
```

### Vacuum

```bash
# Analyze and optimize
psql -U postgres -d spatialvortex -c "VACUUM ANALYZE;"
```

### Monitor Size

```sql
SELECT 
    pg_size_pretty(pg_database_size('spatialvortex')) as database_size,
    pg_size_pretty(pg_total_relation_size('flux_matrices')) as matrices_size,
    pg_size_pretty(pg_total_relation_size('inference_log')) as log_size;
```

## Troubleshooting

### Connection Failed

**Error**: `Connection failed: connection to server failed`

**Solutions**:
1. Check PostgreSQL is running: `pg_isready`
2. Verify connection string format
3. Check firewall rules (port 5432)
4. Verify user permissions

### Schema Already Exists

**Error**: `relation already exists`

**Solution**: This is safe to ignore - the schema uses `IF NOT EXISTS` clauses.

### Permission Denied

**Error**: `permission denied for table flux_matrices`

**Solution**:
```sql
GRANT ALL PRIVILEGES ON DATABASE spatialvortex TO your_user;
GRANT ALL ON ALL TABLES IN SCHEMA public TO your_user;
```

## Testing

Run integration tests (requires running PostgreSQL):

```bash
# Set test database URL
export TEST_DATABASE_URL="host=localhost user=postgres dbname=spatialvortex_test"

# Run tests
cargo test --features database_tests
```

## Docker Support

Quick setup with Docker:

```bash
# Start PostgreSQL container
docker run --name spatialvortex-db \
  -e POSTGRES_PASSWORD=secret \
  -e POSTGRES_DB=spatialvortex \
  -p 5432:5432 \
  -d postgres:15

# Run schema
docker exec -i spatialvortex-db psql -U postgres -d spatialvortex < database/schema.sql
```

Connection string:
```
host=localhost port=5432 user=postgres password=secret dbname=spatialvortex
```

## API Reference

See `src/spatial_database.rs` for full API documentation:

- `SpatialDatabase::new(url)` - Create connection pool
- `initialize_schema()` - Set up tables/indexes
- `store_matrix(matrix)` - Upsert FluxMatrix
- `get_matrix_by_subject(subject)` - Retrieve by subject
- `get_matrix_by_id(id)` - Retrieve by UUID
- `delete_matrix(id)` - Remove matrix
- `get_all_subjects()` - List all subjects
- `log_inference(...)` - Record inference operation
- `get_statistics()` - Get database stats
- `health_check()` - Verify connection
