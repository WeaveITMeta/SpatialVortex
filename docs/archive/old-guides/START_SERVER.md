# Quick Start - SpatialVortex Server

## 📋 **Prerequisites**

1. ✅ **Rust** - Already installed
2. ⏳ **Redis** - See [REDIS_SETUP_WINDOWS.md](docs/REDIS_SETUP_WINDOWS.md)
3. 🔧 **PostgreSQL** (optional) - For database features

---

## 🚀 **Start Server (3 Steps)**

### **Step 1: Install Redis**

**Easiest - Chocolatey + Memurai**:
```powershell
choco install memurai-developer -y
```

**Or see**: `docs/REDIS_SETUP_WINDOWS.md` for alternatives

---

### **Step 2: Verify Redis is Running**

```powershell
redis-cli ping
# Or if using Memurai:
memurai-cli ping

# Expected output: PONG
```

---

### **Step 3: Start SpatialVortex Server**

```powershell
cargo run --bin spatial-vortex -- --host 127.0.0.1 --port 7000
```

**Server will start on**: `http://127.0.0.1:7000`

---

## 🎯 **Testing the Server**

### **Health Check**:
```powershell
# In another terminal
curl http://127.0.0.1:7000/health

# Or in browser:
Start-Process "http://127.0.0.1:7000"
```

### **API Endpoints**:
- `GET /health` - Health check
- `POST /inference` - Run inference
- `POST /matrices` - Create flux matrix
- `GET /matrices/:id` - Get matrix by ID

---

## 🔧 **Configuration**

### **Environment Variables** (optional):

Create `.env` file in project root:
```env
# Redis
REDIS_URL=redis://127.0.0.1:6379

# Database (optional)
DATABASE_URL=postgresql://localhost/spatial_vortex

# AI Integration (optional)
AI_API_KEY=your_key_here
AI_MODEL_ENDPOINT=http://localhost:8000
```

### **Command Line Options**:
```powershell
cargo run --bin spatial-vortex -- --help

# Options:
#   --host <HOST>          Server host [default: 127.0.0.1]
#   -p, --port <PORT>      Server port [default: 7000]
#   -d, --database-url <URL>     Database URL [env: DATABASE_URL]
#   -r, --redis-url <URL>        Redis URL [env: REDIS_URL]
#   -a, --ai-api-key <KEY>       AI API key [env: AI_API_KEY]
#   --ai-endpoint <URL>          AI endpoint [env: AI_MODEL_ENDPOINT]
#   --init-db                    Initialize database schema
#   --bootstrap                  Load example matrices
```

---

## 🎨 **Alternative: Just Run Visualizations**

If you don't need the server, generate visualizations instead:

```powershell
# 2D Visualization (no Redis needed)
cargo run --example render_flux_2d

# Output: flux_matrix_2d.png (beautiful!)
```

---

## 📊 **What's Next?**

### **After Server Starts**:
1. ✅ Test health endpoint
2. ✅ Send inference requests
3. ✅ Create flux matrices
4. ✅ Run benchmarks: `cargo bench`
5. ✅ Generate visualizations

### **With Redis Running**:
- ✅ Caching enabled (24hr TTL)
- ✅ Fast matrix lookups
- ✅ Session management
- ✅ Real-time updates

---

## 🐛 **Troubleshooting**

### **"Redis connection refused"**:
```powershell
# Check if Redis is running
tasklist | findstr redis
# Or for Memurai:
tasklist | findstr memurai

# Restart Redis
net stop Memurai
net start Memurai
```

### **"Port 7000 already in use"**:
```powershell
# Find what's using it
netstat -ano | findstr 7000

# Kill the process
taskkill /PID <PID> /F

# Or use different port
cargo run --bin spatial-vortex -- --port 7001
```

### **Compilation errors**:
```powershell
# Clean and rebuild
cargo clean
cargo build --bin spatial-vortex
```

---

## 🏆 **Success!**

When server starts, you'll see:
```
✅ Connected to Redis at redis://127.0.0.1:6379
📊 Starting Spatial Vortex server on 127.0.0.1:7000
🚀 Server running!
```

**Press Ctrl+C to stop the server**

---

## 📚 **Documentation**

- **Setup**: This file
- **Redis**: `docs/REDIS_SETUP_WINDOWS.md`
- **Architecture**: `docs/COMPLETION_SUMMARY.md`
- **API**: Coming soon!
