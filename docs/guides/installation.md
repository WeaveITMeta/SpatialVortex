# SpatialVortex Production Setup Guide

**Status**: Follow these steps to get to 95%+ production ready

---

## 📋 Prerequisites

Before running SpatialVortex in production, you need:

### 1. PostgreSQL Database ✅
```bash
# Install PostgreSQL 15+ (choose one method)

# macOS (Homebrew)
brew install postgresql@15
brew services start postgresql@15

# Ubuntu/Debian
sudo apt-get install postgresql-15
sudo systemctl start postgresql

# Docker
docker run --name spatialvortex-db \
  -e POSTGRES_PASSWORD=yourpassword \
  -e POSTGRES_DB=spatialvortex \
  -p 5432:5432 \
  -d postgres:15
```

Create the database:
```sql
CREATE DATABASE spatialvortex;
CREATE USER spatialvortex WITH ENCRYPTED PASSWORD 'yourpassword';
GRANT ALL PRIVILEGES ON DATABASE spatialvortex TO spatialvortex;
```

### 2. Redis Cache ✅
```bash
# Install Redis (choose one method)

# macOS (Homebrew)
brew install redis
brew services start redis

# Ubuntu/Debian
sudo apt-get install redis-server
sudo systemctl start redis

# Docker
docker run --name spatialvortex-redis \
  -p 6379:6379 \
  -d redis:7-alpine
```

### 3. ONNX Model (Optional - for ML inference) ✅
```bash
# Download sentence-transformers model
mkdir -p models/all-MiniLM-L6-v2
cd models/all-MiniLM-L6-v2

# Download from HuggingFace
wget https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/main/model.onnx
wget https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/main/tokenizer.json

# Or use Python to export
python3 -m pip install sentence-transformers onnx
python3 -c "
from sentence_transformers import SentenceTransformer
model = SentenceTransformer('sentence-transformers/all-MiniLM-L6-v2')
model.save('./models/all-MiniLM-L6-v2')
"
```

---

## 🔧 Configuration

### 1. Copy Example Config
```bash
cp config.toml.example config.toml
```

### 2. Edit config.toml

**Required Changes**:
```toml
[database]
url = "postgresql://spatialvortex:yourpassword@localhost:5432/spatialvortex"

[cache]
url = "redis://127.0.0.1:6379/"

[confidence_lake]
# Generate encryption key:
# openssl rand -hex 32
encryption_key = "your-generated-key-here"
```

**Optional (for ML features)**:
```toml
[onnx]
model_path = "./models/all-MiniLM-L6-v2/model.onnx"
tokenizer_path = "./models/all-MiniLM-L6-v2/tokenizer.json"
```

### 3. Or Use Environment Variables

Create `.env` file:
```bash
# Server
SPATIALVORTEX_SERVER_HOST=0.0.0.0
SPATIALVORTEX_SERVER_PORT=8080

# Database
DATABASE_URL=postgresql://spatialvortex:yourpassword@localhost:5432/spatialvortex

# Cache
REDIS_URL=redis://127.0.0.1:6379/

# Encryption
SPATIALVORTEX_LAKE_ENCRYPTION_KEY=$(openssl rand -hex 32)

# ONNX (optional)
SPATIALVORTEX_ONNX_MODEL_PATH=./models/all-MiniLM-L6-v2/model.onnx
SPATIALVORTEX_ONNX_TOKENIZER_PATH=./models/all-MiniLM-L6-v2/tokenizer.json

# Logging
RUST_LOG=info
```

---

## 🚀 Running the Server

### Development
```bash
# With all features
cargo run --bin api_server --features onnx,lake,voice

# Without ML
cargo run --bin api_server --features lake,voice

# Minimal
cargo run --bin api_server
```

### Production
```bash
# Build release binary
cargo build --bin api_server --features onnx,lake,voice --release

# Run with config file
./target/release/api_server

# Or with environment variables
source .env
./target/release/api_server
```

### Docker
```bash
# Build image
docker build -t spatialvortex-api .

# Run container
docker run -d \
  --name spatialvortex-api \
  -p 8080:8080 \
  -v $(pwd)/config.toml:/app/config.toml \
  -v $(pwd)/models:/app/models \
  -v $(pwd)/data:/app/data \
  --env-file .env \
  spatialvortex-api
```

---

## ✅ Verification

### 1. Check Services
```bash
# PostgreSQL
psql -h localhost -U spatialvortex -d spatialvortex -c "SELECT 1"

# Redis
redis-cli ping

# Should return PONG
```

### 2. Test API
```bash
# Health check
curl http://localhost:8080/api/v1/health

# Should return:
# {"status":"healthy","version":"0.1.0",...}
```

### 3. Test ML Inference (if ONNX enabled)
```bash
curl -X POST http://localhost:8080/api/v1/ml/embed \
  -H "Content-Type: application/json" \
  -d '{"text":"Hello world","include_sacred_geometry":true}'

# Should return embedding + ELP channels + signal strength
```

### 4. Test Confidence Lake (if enabled)
```bash
curl http://localhost:8080/api/v1/storage/confidence-lake/status

# Should return:
# {"status":"ready","encryption_enabled":true,...}
```

---

## 📊 Production Checklist

### Infrastructure ✅
- [ ] PostgreSQL running and accessible
- [ ] Redis running and accessible
- [ ] ONNX models downloaded (if using ML)
- [ ] Data directory created with proper permissions
- [ ] Encryption key generated and secured

### Configuration ✅
- [ ] config.toml created from example
- [ ] Database URL configured
- [ ] Redis URL configured
- [ ] Encryption key configured (if using lake)
- [ ] ONNX paths configured (if using ML)
- [ ] Environment variables set (production)

### Security ✅
- [ ] Encryption key stored securely (not in git)
- [ ] Database password secured
- [ ] API authentication configured (TODO)
- [ ] HTTPS/TLS configured (TODO)
- [ ] Firewall rules configured

### Deployment ✅
- [ ] Server binary built in release mode
- [ ] Systemd service created (Linux)
- [ ] Docker image built (if using containers)
- [ ] Reverse proxy configured (nginx/traefik)
- [ ] Monitoring configured (prometheus/grafana)

### Testing ✅
- [ ] Health endpoint responds
- [ ] Database connection works
- [ ] Redis cache works
- [ ] ML inference works (if enabled)
- [ ] Confidence Lake works (if enabled)
- [ ] All endpoints return valid responses

---

## 🔐 Security Best Practices

### 1. Encryption Key Management
```bash
# Generate strong key
openssl rand -hex 32

# Store in secure location
echo "SPATIALVORTEX_LAKE_ENCRYPTION_KEY=$(openssl rand -hex 32)" >> /etc/spatialvortex/.env
chmod 600 /etc/spatialvortex/.env

# Never commit to git!
echo ".env" >> .gitignore
echo "config.toml" >> .gitignore
```

### 2. Database Security
```sql
-- Create read-only user for reporting
CREATE USER spatialvortex_readonly WITH ENCRYPTED PASSWORD 'readonly_pass';
GRANT CONNECT ON DATABASE spatialvortex TO spatialvortex_readonly;
GRANT SELECT ON ALL TABLES IN SCHEMA public TO spatialvortex_readonly;

-- Enable SSL connections
ALTER SYSTEM SET ssl = on;
```

### 3. Redis Security
```bash
# Edit redis.conf
requirepass your-redis-password
bind 127.0.0.1  # Only local connections
```

---

## 🐛 Troubleshooting

### Server won't start
```bash
# Check config validity
cargo run --bin api_server -- --check-config

# Check logs
RUST_LOG=debug cargo run --bin api_server
```

### Database connection fails
```bash
# Test connection manually
psql postgresql://spatialvortex:password@localhost:5432/spatialvortex

# Check if PostgreSQL is running
sudo systemctl status postgresql

# Check firewall
sudo ufw status
```

### Redis connection fails
```bash
# Test connection
redis-cli ping

# Check if Redis is running
sudo systemctl status redis

# Check port
netstat -tulpn | grep 6379
```

### ONNX model not found
```bash
# Verify path
ls -la ./models/all-MiniLM-L6-v2/

# Should show:
# model.onnx
# tokenizer.json

# Re-download if missing
# (see Prerequisites section)
```

---

## 📝 Next Steps After Setup

Once your server is running:

1. **Test all endpoints** - See `docs/API_ENDPOINTS.md`
2. **Configure monitoring** - Prometheus + Grafana
3. **Set up CI/CD** - GitHub Actions / Jenkins
4. **Load test** - Use k6 or Apache Bench
5. **Add authentication** - JWT or API keys
6. **Configure HTTPS** - Let's Encrypt + nginx

---

## 💡 Quick Start (Development)

Fastest way to get started:

```bash
# 1. Start services with Docker Compose
docker-compose up -d

# 2. Generate encryption key
export SPATIALVORTEX_LAKE_ENCRYPTION_KEY=$(openssl rand -hex 32)

# 3. Run server
cargo run --bin api_server --features lake

# 4. Test
curl http://localhost:8080/api/v1/health
```

---

## 📚 Additional Resources

- **API Documentation**: `docs/API_ENDPOINTS.md`
- **Architecture**: `docs/architecture/`
- **Examples**: `examples/`
- **Discord**: [link]
- **GitHub**: https://github.com/WeaveSolutions/SpatialVortex

---

**Questions?** Open an issue on GitHub or check the troubleshooting section.
