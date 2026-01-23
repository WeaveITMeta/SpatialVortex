# Redis Setup Guide for Windows

**Required for**: SpatialVortex REST API Server

---

## 🎯 **Recommended: Memurai (Easiest)**

Memurai is Redis-compatible and works natively on Windows.

### **Install via Chocolatey**:
```powershell
# Open PowerShell as Administrator
choco install memurai-developer -y
```

### **Or Download Directly**:
1. Visit: https://www.memurai.com/get-memurai
2. Download Memurai Developer (Free)
3. Run installer
4. Memurai will start automatically as a Windows service

### **Verify Installation**:
```powershell
memurai-cli ping
# Should return: PONG
```

---

## 🐧 **Alternative: WSL (Windows Subsystem for Linux)**

### **Install WSL and Redis**:
```powershell
# In PowerShell (as Administrator)
wsl --install

# Restart computer if prompted

# After restart, open WSL terminal
wsl

# Inside WSL
sudo apt update
sudo apt install redis-server -y

# Start Redis
sudo service redis-server start

# Verify
redis-cli ping
# Should return: PONG
```

### **Keep WSL Redis Running**:
```bash
# In WSL terminal
sudo service redis-server start

# To auto-start on WSL launch, add to ~/.bashrc:
echo "sudo service redis-server start" >> ~/.bashrc
```

---

## 🐳 **Alternative: Docker**

If you have Docker Desktop installed:

```powershell
# Start Redis container
docker run -d `
  --name spatial-vortex-redis `
  -p 6379:6379 `
  redis:latest

# Verify
docker exec -it spatial-vortex-redis redis-cli ping
# Should return: PONG
```

### **Stop/Start**:
```powershell
docker stop spatial-vortex-redis
docker start spatial-vortex-redis
```

---

## 🔧 **Alternative: Manual Windows Port**

### **Download**:
1. Go to: https://github.com/tporadowski/redis/releases
2. Download latest `Redis-x64-*.zip`
3. Extract to `C:\Redis`

### **Run Redis**:
```powershell
cd C:\Redis
.\redis-server.exe
```

### **Test**:
```powershell
# In another terminal
cd C:\Redis
.\redis-cli.exe ping
# Should return: PONG
```

---

## ✅ **Verify Redis is Running**

### **Test Connection**:
```powershell
# Using redis-cli (or memurai-cli)
redis-cli ping

# Or using telnet
telnet localhost 6379
# Type: PING
# Should see: +PONG
```

### **Check Port**:
```powershell
netstat -an | findstr 6379
# Should show: TCP    0.0.0.0:6379    0.0.0.0:0    LISTENING
```

---

## 🚀 **Start SpatialVortex Server**

Once Redis is running:

```powershell
cd E:\Libraries\SpatialVortex

# Option 1: Use default Redis location (localhost:6379)
cargo run --bin spatial-vortex -- --host 127.0.0.1 --port 7000

# Option 2: Specify custom Redis URL
$env:REDIS_URL="redis://127.0.0.1:6379"
cargo run --bin spatial-vortex -- --host 127.0.0.1 --port 7000
```

---

## 🔍 **Troubleshooting**

### **"Connection Refused" Error**:
```powershell
# Check if Redis is running
tasklist | findstr redis

# Check port
netstat -an | findstr 6379

# Restart Redis
# (Method depends on installation - service, docker, or manual)
```

### **Permission Denied**:
```powershell
# Run PowerShell as Administrator for installation
```

### **Port Already in Use**:
```powershell
# Find what's using port 6379
netstat -ano | findstr 6379

# Kill the process (replace PID)
taskkill /PID <PID> /F
```

---

## 📊 **Redis Management**

### **View Data**:
```powershell
redis-cli

# Inside redis-cli:
KEYS *                  # List all keys
GET <key>               # Get value
FLUSHALL                # Clear all data (careful!)
INFO                    # Server info
```

### **Monitor Activity**:
```powershell
redis-cli MONITOR
# Shows all commands in real-time
```

---

## 🎯 **Recommended for Development**

**Best Option**: **Memurai** (easiest, native Windows, runs as service)

**Pros**:
- ✅ Native Windows application
- ✅ Runs as Windows service (auto-start)
- ✅ Redis-compatible (no code changes)
- ✅ Free for development
- ✅ GUI management tool included

**Quick Install**:
```powershell
choco install memurai-developer -y
```

**Done!** Redis will be running on `localhost:6379`
