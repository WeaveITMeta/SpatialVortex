# 🚀 Task Management API - Quick Start Guide

**Date**: November 4, 2025  
**Feature**: Multi-Step Task Execution via REST API

---

## 📋 Overview

The Task Management API allows Vortex to accept **multiple instructions** and execute them systematically, tracking progress and persisting state across sessions.

---

## ⚡ Quick Start

### 1. Start the Server

```powershell
cd e:\Libraries\SpatialVortex
cargo run --release --bin api_server
```

**Server will start at**: `http://localhost:7000`

---

### 2. Create a Task Queue

**Request:**
```bash
curl -X POST http://localhost:7000/api/v1/tasks/create \
  -H "Content-Type: application/json" \
  -d '{
    "session_id": "demo_session_123",
    "instructions": "1. Write a binary search function in Rust\n2. Explain time complexity\n3. Write unit tests"
  }'
```

**Response:**
```json
{
  "session_id": "demo_session_123",
  "task_ids": [
    "task_1_1730727480",
    "task_2_1730727481",
    "task_3_1730727482"
  ],
  "total_tasks": 3,
  "tasks": [
    {
      "id": "task_1_1730727480",
      "instruction": "Write a binary search function in Rust",
      "task_type": "Coding",
      "status": "Pending"
    },
    {
      "id": "task_2_1730727481",
      "instruction": "Explain time complexity",
      "task_type": "Analysis",
      "status": "Pending"
    },
    {
      "id": "task_3_1730727482",
      "instruction": "Write unit tests",
      "task_type": "Coding",
      "status": "Pending"
    }
  ]
}
```

✅ **3 tasks created and queued!**

---

### 3. Execute Next Task

**Request:**
```bash
curl -X POST http://localhost:7000/api/v1/tasks/execute \
  -H "Content-Type: application/json" \
  -d '{
    "session_id": "demo_session_123"
  }'
```

**Response:**
```json
{
  "task_id": "task_1_1730727480",
  "instruction": "Write a binary search function in Rust",
  "status": "Completed",
  "result": "fn binary_search<T: Ord>(arr: &[T], target: &T) -> Option<usize> {\n    let mut low = 0;\n    let mut high = arr.len();\n    \n    while low < high {\n        let mid = (low + high) / 2;\n        match arr[mid].cmp(target) {\n            std::cmp::Ordering::Equal => return Some(mid),\n            std::cmp::Ordering::Less => low = mid + 1,\n            std::cmp::Ordering::Greater => high = mid,\n        }\n    }\n    None\n}",
  "error": null,
  "progress": {
    "total": 3,
    "completed": 1,
    "failed": 0,
    "pending": 2,
    "all_done": false,
    "percentage": 33.33
  }
}
```

✅ **Task 1 complete! (33% done)**

---

### 4. Check Progress

**Request:**
```bash
curl http://localhost:7000/api/v1/tasks/progress?session_id=demo_session_123
```

**Response:**
```json
{
  "total": 3,
  "completed": 1,
  "failed": 0,
  "pending": 2,
  "all_done": false,
  "percentage": 33.33
}
```

---

### 5. Execute All Remaining Tasks

**Request:**
```bash
curl -X POST http://localhost:7000/api/v1/tasks/execute-all \
  -H "Content-Type: application/json" \
  -d '{
    "session_id": "demo_session_123"
  }'
```

**Response:**
```json
{
  "session_id": "demo_session_123",
  "total_executed": 2,
  "results": [
    {
      "task_id": "task_2_1730727481",
      "instruction": "Explain time complexity",
      "status": "Completed",
      "result": "## Time Complexity of Binary Search\n\n**Best Case**: O(1) - Target is at the middle\n**Average Case**: O(log n) - Halving search space each iteration\n**Worst Case**: O(log n) - Must search until single element remains\n\n**Space Complexity**: O(1) - Iterative implementation uses constant space",
      "error": null,
      "progress": {
        "total": 3,
        "completed": 2,
        "failed": 0,
        "pending": 1,
        "all_done": false,
        "percentage": 66.67
      }
    },
    {
      "task_id": "task_3_1730727482",
      "instruction": "Write unit tests",
      "status": "Completed",
      "result": "#[cfg(test)]\nmod tests {\n    use super::*;\n\n    #[test]\n    fn test_binary_search_found() {\n        let arr = [1, 3, 5, 7, 9, 11];\n        assert_eq!(binary_search(&arr, &5), Some(2));\n    }\n\n    #[test]\n    fn test_binary_search_not_found() {\n        let arr = [1, 3, 5, 7, 9];\n        assert_eq!(binary_search(&arr, &6), None);\n    }\n\n    #[test]\n    fn test_binary_search_empty() {\n        let arr: [i32; 0] = [];\n        assert_eq!(binary_search(&arr, &1), None);\n    }\n}",
      "error": null,
      "progress": {
        "total": 3,
        "completed": 3,
        "failed": 0,
        "pending": 0,
        "all_done": true,
        "percentage": 100.0
      }
    }
  ]
}
```

✅ **All tasks complete! (100%)**

---

### 6. View All Tasks

**Request:**
```bash
curl http://localhost:7000/api/v1/tasks/list?session_id=demo_session_123
```

**Response:**
```json
{
  "session_id": "demo_session_123",
  "total": 3,
  "tasks": [
    {
      "id": "task_1_1730727480",
      "instruction": "Write a binary search function in Rust",
      "task_type": "Coding",
      "status": "Completed"
    },
    {
      "id": "task_2_1730727481",
      "instruction": "Explain time complexity",
      "task_type": "Analysis",
      "status": "Completed"
    },
    {
      "id": "task_3_1730727482",
      "instruction": "Write unit tests",
      "task_type": "Coding",
      "status": "Completed"
    }
  ]
}
```

---

## 🎯 Supported Instruction Formats

### Numbered Lists
```
1. Task one
2. Task two
3. Task three
```

### Bullet Lists
```
- Task one
- Task two
- Task three
```

### Natural Language
```
First, create a database schema; then write migration scripts; finally, test the migrations
```

---

## 🤖 Task Type Detection

Vortex automatically detects task type from keywords:

| Keywords | Type | Agent Used |
|----------|------|------------|
| "write code", "implement", "function" | **Coding** | EnhancedCodingAgent |
| "analyze", "explain", "what is" | **Analysis** | ThinkingAgent |
| "research", "find", "look up" | **Research** | ThinkingAgent (RAG) |
| "think", "reason", "solve" | **Reasoning** | ThinkingAgent (CoT) |
| Default | **General** | ThinkingAgent |

---

## 📊 Response Structure

### Task Execution Result

```typescript
{
  task_id: string,         // Unique task ID
  instruction: string,     // Original instruction
  status: "Completed" | "Failed",
  result: string | null,   // Task output (code, text, etc.)
  error: string | null,    // Error message if failed
  progress: {
    total: number,         // Total tasks in queue
    completed: number,     // Tasks finished
    failed: number,        // Tasks that errored
    pending: number,       // Tasks not started
    all_done: boolean,     // True when queue complete
    percentage: number     // Completion percentage (0-100)
  }
}
```

---

## 🔥 Advanced Examples

### Example 1: Full Stack Development

```bash
curl -X POST http://localhost:7000/api/v1/tasks/create \
  -H "Content-Type: application/json" \
  -d '{
    "session_id": "fullstack_dev",
    "instructions": "1. Create a User model with authentication fields\n2. Write REST API endpoints for user CRUD\n3. Add JWT authentication middleware\n4. Write integration tests\n5. Document the API with OpenAPI specs"
  }'
```

**Result**: 5 tasks queued, executed sequentially by appropriate agents.

---

### Example 2: Data Analysis Pipeline

```bash
curl -X POST http://localhost:7000/api/v1/tasks/create \
  -H "Content-Type: application/json" \
  -d '{
    "session_id": "data_pipeline",
    "instructions": "- Load dataset from CSV\n- Clean and normalize data\n- Perform statistical analysis\n- Generate visualization code\n- Write summary report"
  }'
```

**Result**: Mixed coding and analysis tasks, executed in order.

---

### Example 3: Research & Implementation

```bash
curl -X POST http://localhost:7000/api/v1/tasks/create \
  -H "Content-Type: application/json" \
  -d '{
    "session_id": "research_impl",
    "instructions": "Research best practices for API rate limiting; Implement a token bucket algorithm in Rust; Write benchmarks for the implementation"
  }'
```

**Result**: Research → Code → Test workflow.

---

## ⚡ PowerShell Examples

### Create & Execute All (Windows)

```powershell
# Create tasks
$createResponse = Invoke-RestMethod -Method Post -Uri "http://localhost:7000/api/v1/tasks/create" -ContentType "application/json" -Body (@{
    session_id = "ps_session"
    instructions = "1. Write a quicksort in Python`n2. Analyze its performance`n3. Compare with merge sort"
} | ConvertTo-Json)

Write-Host "Created $($createResponse.total_tasks) tasks"

# Execute all
$execResponse = Invoke-RestMethod -Method Post -Uri "http://localhost:7000/api/v1/tasks/execute-all" -ContentType "application/json" -Body (@{
    session_id = "ps_session"
} | ConvertTo-Json)

Write-Host "Completed $($execResponse.total_executed) tasks"

# Show results
foreach ($result in $execResponse.results) {
    Write-Host "`n=== $($result.instruction) ===" -ForegroundColor Cyan
    Write-Host $result.result
}
```

---

## 🎯 Use Cases

### ✅ Software Development
- Multi-file code generation
- Feature implementation pipelines
- Bug fix workflows
- Testing & documentation

### ✅ Data Science
- Data cleaning pipelines
- Analysis workflows
- Model training steps
- Report generation

### ✅ Research & Learning
- Topic research
- Implementation examples
- Comparative analysis
- Tutorial creation

### ✅ DevOps
- Infrastructure setup
- Deployment pipelines
- Configuration management
- Monitoring setup

---

## 🚀 Production Tips

### Session Management
- Use meaningful session IDs (e.g., `user_123_project_abc`)
- Sessions persist across server restarts (in-memory)
- Clean up old sessions periodically

### Error Handling
- Check `error` field in responses
- Failed tasks don't block queue (move to next)
- Review failed tasks via `/api/v1/tasks/list`

### Performance
- Tasks execute sequentially (prevents resource contention)
- Use `/execute-all` for batch processing
- Monitor progress with `/progress` endpoint

---

## 📖 Next Steps

1. **Test the API** - Try the examples above
2. **Integrate with Frontend** - Build UI for task management
3. **Add Webhooks** - Get notified when tasks complete
4. **Parallel Execution** - Coming soon for independent tasks

---

**Feature Status**: ✅ Production Ready  
**Last Updated**: November 4, 2025  
**Documentation**: `TASK_MANAGEMENT.md`  
**Source**: `src/agents/task_manager.rs`, `src/ai/task_api.rs`
