# 🚀 Kubernetes Deployment Guide for SpatialVortex

**Complete Step-by-Step Guide for Production Deployment**

---

## 📋 Prerequisites

Before deploying SpatialVortex to Kubernetes, ensure you have:

- ✅ Kubernetes cluster (v1.25+)
- ✅ kubectl configured
- ✅ Docker registry access
- ✅ Helm 3 (optional, for package management)
- ✅ cert-manager (for TLS certificates)
- ✅ NVIDIA GPU nodes (optional, for ML acceleration)

---

## 🔧 Step 1: Build and Push Docker Image

### 1.1 Build the Docker Image

```bash
# Clone the repository
git clone https://github.com/WeaveSolutions/SpatialVortex.git
cd SpatialVortex

# Build with all features
docker build -t spatialvortex:latest \
  --build-arg FEATURES="voice,lake,onnx" .

# Tag for your registry
docker tag spatialvortex:latest your-registry.io/spatialvortex:v1.0.0
```

### 1.2 Push to Registry

```bash
# Login to your registry
docker login your-registry.io

# Push the image
docker push your-registry.io/spatialvortex:v1.0.0
```

---

## 🗄️ Step 2: Prepare Persistent Storage

### 2.1 Create Storage Classes (if needed)

```yaml
# storage-class.yaml
apiVersion: storage.k8s.io/v1
kind: StorageClass
metadata:
  name: fast-ssd
provisioner: kubernetes.io/aws-ebs  # Or your cloud provider
parameters:
  type: gp3
  iops: "3000"
  throughput: "125"
reclaimPolicy: Retain
volumeBindingMode: WaitForFirstConsumer
```

Apply:
```bash
kubectl apply -f storage-class.yaml
```

### 2.2 Upload ML Models to Persistent Volume

```bash
# Create a temporary pod to upload models
kubectl run model-uploader --image=busybox --restart=Never \
  --overrides='
{
  "spec": {
    "containers": [{
      "name": "uploader",
      "image": "busybox",
      "command": ["sleep", "3600"],
      "volumeMounts": [{
        "name": "models",
        "mountPath": "/models"
      }]
    }],
    "volumes": [{
      "name": "models",
      "persistentVolumeClaim": {
        "claimName": "spatialvortex-models"
      }
    }]
  }
}'

# Copy models
kubectl cp ./models/sentence-transformers model-uploader:/models/
kubectl cp ./models/tokenizers model-uploader:/models/

# Cleanup
kubectl delete pod model-uploader
```

---

## 🔐 Step 3: Configure Secrets

### 3.1 Generate Encryption Key

```bash
# Generate 32-byte key for AES-256
openssl rand -base64 32 > encryption.key

# Create Kubernetes secret
kubectl create secret generic spatialvortex-secrets \
  --from-file=encryption-key=encryption.key \
  --from-literal=database-url="sqlite:///data/confidence_lake.db" \
  -n spatialvortex
```

### 3.2 Configure TLS (Production)

```bash
# Install cert-manager
kubectl apply -f https://github.com/cert-manager/cert-manager/releases/download/v1.13.0/cert-manager.yaml

# Create ClusterIssuer for Let's Encrypt
cat <<EOF | kubectl apply -f -
apiVersion: cert-manager.io/v1
kind: ClusterIssuer
metadata:
  name: letsencrypt-prod
spec:
  acme:
    server: https://acme-v02.api.letsencrypt.org/directory
    email: your-email@example.com
    privateKeySecretRef:
      name: letsencrypt-prod
    solvers:
    - http01:
        ingress:
          class: nginx
EOF
```

---

## 🚀 Step 4: Deploy SpatialVortex

### 4.1 Apply Base Deployment

```bash
# Create namespace
kubectl create namespace spatialvortex

# Apply the deployment
kubectl apply -f kubernetes/deployment.yaml

# Verify pods are running
kubectl get pods -n spatialvortex -w
```

### 4.2 Configure for Your Environment

Edit the deployment to match your setup:

```bash
# Edit the deployment
kubectl edit deployment spatialvortex-asi -n spatialvortex

# Key settings to adjust:
# - image: your-registry.io/spatialvortex:v1.0.0
# - replicas: 3 (or based on load)
# - resources: adjust CPU/memory based on nodes
# - environment variables for features
```

### 4.3 GPU Support (Optional)

For NVIDIA GPU acceleration:

```yaml
# Add to container spec
resources:
  limits:
    nvidia.com/gpu: 1
nodeSelector:
  accelerator: nvidia-tesla-v100
```

---

## 🌐 Step 5: Configure Networking

### 5.1 Internal Service

The base deployment creates a LoadBalancer service. For cloud providers:

**AWS**:
```yaml
metadata:
  annotations:
    service.beta.kubernetes.io/aws-load-balancer-type: "nlb"
```

**GCP**:
```yaml
metadata:
  annotations:
    cloud.google.com/load-balancer-type: "Internal"
```

**Azure**:
```yaml
metadata:
  annotations:
    service.beta.kubernetes.io/azure-load-balancer-internal: "true"
```

### 5.2 Configure Ingress

```bash
# Install NGINX Ingress Controller
kubectl apply -f https://raw.githubusercontent.com/kubernetes/ingress-nginx/controller-v1.8.2/deploy/static/provider/cloud/deploy.yaml

# Update ingress host
kubectl edit ingress spatialvortex-ingress -n spatialvortex
# Change: api.spatialvortex.ai → your-domain.com
```

---

## 📊 Step 6: Monitoring & Observability

### 6.1 Prometheus Metrics

```yaml
# prometheus-servicemonitor.yaml
apiVersion: monitoring.coreos.com/v1
kind: ServiceMonitor
metadata:
  name: spatialvortex-metrics
  namespace: spatialvortex
spec:
  selector:
    matchLabels:
      app: spatialvortex
  endpoints:
  - port: metrics
    interval: 30s
```

### 6.2 Grafana Dashboard

Import dashboard ID: `15463` for Rust application metrics

### 6.3 Logging

```bash
# View logs
kubectl logs -f deployment/spatialvortex-asi -n spatialvortex

# Stream all pods
kubectl logs -f -l app=spatialvortex -n spatialvortex --all-containers
```

---

## 🔄 Step 7: Scaling Configuration

### 7.1 Horizontal Pod Autoscaler

The HPA is configured to scale between 3-10 replicas based on:
- CPU utilization > 70%
- Memory utilization > 80%

Monitor scaling:
```bash
kubectl get hpa -n spatialvortex -w
```

### 7.2 Vertical Pod Autoscaler (Optional)

```yaml
apiVersion: autoscaling.k8s.io/v1
kind: VerticalPodAutoscaler
metadata:
  name: spatialvortex-vpa
  namespace: spatialvortex
spec:
  targetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: spatialvortex-asi
  updatePolicy:
    updateMode: "Auto"
```

---

## 🧪 Step 8: Testing the Deployment

### 8.1 Health Checks

```bash
# Port-forward for local testing
kubectl port-forward svc/spatialvortex-service 8080:80 -n spatialvortex

# Test health endpoint
curl http://localhost:8080/health

# Test API
curl -X POST http://localhost:8080/api/v1/process \
  -H "Content-Type: application/json" \
  -d '{"input": "Test sacred geometry", "mode": "fast"}'
```

### 8.2 Voice Pipeline Test

```bash
# Test voice processing (requires voice feature)
curl -X POST http://localhost:8080/api/v1/voice/process \
  -H "Content-Type: application/json" \
  -d '{
    "audio_data": "base64_encoded_audio",
    "sample_rate": 44100
  }'
```

### 8.3 Load Testing

```bash
# Install k6
brew install k6

# Run load test
k6 run --vus 100 --duration 30s tests/k6/load-test.js
```

---

## 🔧 Step 9: Production Checklist

### Security
- [ ] Encryption key rotated and stored in secret manager
- [ ] Network policies configured
- [ ] RBAC roles defined
- [ ] Pod security policies enabled
- [ ] Image vulnerability scanning enabled

### Performance
- [ ] Resource limits tuned based on load testing
- [ ] HPA thresholds validated
- [ ] Database connection pooling configured
- [ ] ONNX session pool size optimized

### Reliability
- [ ] Backup strategy for Confidence Lake
- [ ] Disaster recovery plan documented
- [ ] Multi-zone deployment configured
- [ ] Circuit breakers implemented

### Monitoring
- [ ] Metrics dashboard created
- [ ] Alerts configured for SLOs
- [ ] Log aggregation set up
- [ ] Distributed tracing enabled

---

## 🚨 Step 10: Troubleshooting

### Pod Not Starting

```bash
# Check events
kubectl describe pod <pod-name> -n spatialvortex

# Common issues:
# - ImagePullBackOff: Check registry credentials
# - CrashLoopBackOff: Check logs
# - Pending: Check resource availability
```

### High Memory Usage

```bash
# Check memory consumption
kubectl top pods -n spatialvortex

# Adjust ONNX pool size
kubectl set env deployment/spatialvortex-asi \
  ONNX_POOL_SIZE=2 -n spatialvortex
```

### Slow Performance

```bash
# Check CPU throttling
kubectl describe nodes | grep -A5 "Allocated resources"

# Scale up if needed
kubectl scale deployment spatialvortex-asi --replicas=5 -n spatialvortex
```

---

## 📚 Additional Resources

- **Helm Chart**: Coming soon at `helm.spatialvortex.ai`
- **Operator**: Kubernetes operator for automated management (roadmap)
- **Support**: GitHub Issues or community Discord

---

## 🎯 Quick Start Commands

```bash
# Complete deployment in one script
#!/bin/bash

# Create namespace
kubectl create namespace spatialvortex

# Create secrets
kubectl create secret generic spatialvortex-secrets \
  --from-literal=encryption-key=$(openssl rand -base64 32) \
  --from-literal=database-url="sqlite:///data/confidence_lake.db" \
  -n spatialvortex

# Deploy
kubectl apply -f https://raw.githubusercontent.com/WeaveSolutions/SpatialVortex/main/kubernetes/deployment.yaml

# Wait for rollout
kubectl rollout status deployment/spatialvortex-asi -n spatialvortex

# Get external IP
kubectl get svc spatialvortex-service -n spatialvortex
```

---

**Production Ready!** Your SpatialVortex ASI system is now running on Kubernetes with:
- ✅ Auto-scaling
- ✅ High availability
- ✅ Persistent storage
- ✅ TLS encryption
- ✅ Monitoring ready

For questions or issues, please refer to our [GitHub repository](https://github.com/WeaveSolutions/SpatialVortex).
