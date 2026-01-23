# VMAI Quick Start Guide
## Get Running with Virtual Machine AI in 30 Minutes

**Prerequisites**: SpatialVortex ASI system installed

---

## 🚀 Installation

### Step 1: Install VMAI Dependencies

```bash
# Install containerization tools
cargo install --git https://github.com/containers/youki youki

# Install Microsoft Omni Parser (Python)
pip install omniparser-sdk

# Install resource management tools
sudo apt-get install cgroup-tools
```

### Step 2: Build VMAI

```bash
cd SpatialVortex
cargo build --release --features vmai
```

---

## 🎯 Basic Usage

### Example 1: Create Simple VM

```rust
use spatialvortex::vmai::{VMAIHypervisor, VMAIConfig, VMResources};

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Create hypervisor
    let mut hypervisor = VMAIHypervisor::new().await?;
    
    // 2. Configure VM
    let config = VMAIConfig {
        position_hint: Some(6),  // Analytical tasks
        resources: VMResources {
            cpu_cores: 4,
            memory_mb: 8192,
            gpu_allocation: None,
            disk_space_gb: 50,
        },
        parser_capabilities: ParserCapabilities::text_only(),
        tool_capabilities: vec!["web_search", "math"],
        sandbox_level: SandboxLevel::Standard,
    };
    
    // 3. Create VM
    let vm_id = hypervisor.create_instance(config).await?;
    println!("Created VM: {}", vm_id);
    
    // 4. Execute task
    let result = hypervisor.execute_on(vm_id, AITask {
        input: MultimodalInput::Text("What is 2+2?".to_string()),
        timeout: Duration::from_secs(10),
    }).await?;
    
    println!("Result: {:?}", result);
    
    // 5. Shutdown
    hypervisor.shutdown_instance(vm_id).await?;
    
    Ok(())
}
```

---

## 📋 Common Patterns

### Pattern 1: Multi-VM RAG

```rust
// Create retriever at position 3 (creative)
let retriever = hypervisor.create_instance(VMAIConfig {
    position_hint: Some(3),
    resources: VMResources::medium(),
    tool_capabilities: vec!["web_search", "database"],
    ..Default::default()
}).await?;

// Create analyzer at position 6 (analytical)
let analyzer = hypervisor.create_instance(VMAIConfig {
    position_hint: Some(6),
    resources: VMResources::large(),
    tool_capabilities: vec!["code_execution"],
    ..Default::default()
}).await?;

// Create synthesizer at position 9 (completion)
let synthesizer = hypervisor.create_instance(VMAIConfig {
    position_hint: Some(9),
    resources: VMResources::medium(),
    tool_capabilities: vec!["filesystem"],
    ..Default::default()
}).await?;

// Execute pipeline
let docs = retriever.execute(retrieve_task).await?;
let analysis = analyzer.execute(analyze_task(docs)).await?;
let final_result = synthesizer.execute(synthesize_task(analysis)).await?;
```

---

### Pattern 2: Sandboxed Code Execution

```rust
let code_vm = hypervisor.create_instance(VMAIConfig {
    position_hint: Some(5),  // Experimental
    resources: VMResources::small(),
    tool_capabilities: vec!["code_execution"],
    sandbox_level: SandboxLevel::Maximum,  // Maximum isolation
    ..Default::default()
}).await?;

let result = code_vm.execute(AITask {
    input: MultimodalInput::Text(untrusted_code),
    timeout: Duration::from_secs(30),
}).await?;
```

---

## 🔧 Configuration Options

### Resource Presets

```rust
// Small: 2 cores, 2GB RAM
VMResources::small()

// Medium: 4 cores, 8GB RAM
VMResources::medium()

// Large: 8 cores, 16GB RAM
VMResources::large()

// Custom
VMResources {
    cpu_cores: 6,
    memory_mb: 12288,
    gpu_allocation: Some(GpuSlice::partial(0.33)),
    disk_space_gb: 100,
}
```

### Tool Capabilities

```rust
// Common tools
vec!["web_search", "database", "math", "filesystem"]

// Code execution
vec!["code_execution"]

// All tools
ToolCapabilities::all()

// No tools
vec![]
```

### Sandbox Levels

```rust
SandboxLevel::None      // No isolation (testing only)
SandboxLevel::Standard  // Normal isolation
SandboxLevel::Maximum   // Full isolation (untrusted code)
```

---

## 📊 Monitoring

```rust
// Get VM metrics
let metrics = hypervisor.get_metrics(vm_id).await?;
println!("CPU: {}%", metrics.cpu_percent);
println!("Memory: {} MB", metrics.memory_mb);
println!("Tasks completed: {}", metrics.tasks_completed);

// List all VMs
let vms = hypervisor.list_instances().await?;
for vm in vms {
    println!("VM {}: Position {}, Status {:?}", 
             vm.id, vm.position, vm.status);
}
```

---

## 🔒 Security Best Practices

1. **Always use sandboxing** for untrusted input
2. **Set resource limits** to prevent DoS
3. **Use timeout** on all tasks
4. **Whitelist tools** - only enable what's needed
5. **Validate outputs** before trusting results

---

## 🐛 Troubleshooting

### VM Won't Start

```bash
# Check cgroups are enabled
cat /proc/cgroups

# Check container runtime
youki --version
```

### Out of Memory

```rust
// Reduce VM resources
config.resources.memory_mb = 4096;  // Lower limit

// Or increase host memory
// Or reduce number of concurrent VMs
```

### Tool Execution Fails

```rust
// Check tool is registered
let tools = hypervisor.list_available_tools()?;
println!("Available: {:?}", tools);

// Check permissions
config.tool_capabilities = vec!["web_search"];  // Explicit
```

---

## 📚 Next Steps

- Read [VMAI Architecture](../architecture/VMAI_VIRTUAL_MACHINE_AI.md)
- Review [Omni Tool Library](../architecture/VMAI_VIRTUAL_MACHINE_AI.md#omni-tool-library)
- See [Advanced Examples](../examples/vmai/)
- Understand [Sacred Position Assignment](../architecture/SACRED_POSITIONS.md)

---

**Time to First VM**: ~5 minutes  
**Time to Production**: ~30 minutes

