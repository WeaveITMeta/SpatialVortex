# Spatial Vortex REST API Implementation

## Overview

This implementation provides a complete Rust-based REST API for the Spatial Vortex system using Actix-web, featuring:

- **Flux Matrix Engine**: Core 9-node inference system based on the 1,2,4,8,7,5,1 pattern
- **Spatial Database**: PostgreSQL storage for subject matrices and RL data
- **Redis Caching**: High-performance caching for matrix patterns
- **AI Integration**: Grok 4 support for generating subject matrices
- **Bidirectional Inference**: Number ↔ word/meaning processing
- **Moral Alignment**: Constructive vs Destructive semantic reasoning

## Architecture

```
┌─────────────────┐    ┌──────────────────┐     ┌─────────────────┐
│   REST API      │    │  Inference       │     │  Flux Matrix    │
│   (Actix-web)   │───▶│  Engine          │───▶│  Engine         │
└─────────────────┘    └──────────────────┘     └─────────────────┘
         │                       │                       │
         ▼                       ▼                       ▼
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Cache         │    │  Spatial         │    │  AI Integration │
│   (Redis)       │    │  Database        │    │  (Grok 4)       │
└─────────────────┘    │  (PostgreSQL)    │    └─────────────────┘
                       └──────────────────┘
```

## API Endpoints

### Health & Status
- `GET /api/v1/health` - System health check

### Flux Matrix Generation
- `POST /api/v1/flux/matrix/generate` - Generate flux matrix for subject
```json
{
  "subject": "Artificial Intelligence",
  "use_ai_generation": true,
  "sacred_guides_enabled": true
}
```

### Reverse Inference
- `POST /api/v1/inference/reverse` - Process seed numbers to extract meanings (seeds → meanings)
```json
{
  "seed_numbers": [888, 872],
  "subject_filter": "general_intelligence",
  "include_synonyms": true,
  "include_antonyms": true,
  "confidence_threshold": 0.3,
  "use_sacred_guides": true
}
```

### Forward Inference
- `POST /api/v1/inference/forward` - Find seed numbers for target meanings (meanings → seeds)
```json
{
  "target_meanings": ["intelligence", "reasoning", "cognition"],
  "subject_filter": "specific",
  "max_results": 10
}
```

### Matrix Management
- `GET /api/v1/subjects` - List all available subjects
- `GET /api/v1/matrix/{subject}` - Get specific matrix by subject
- `POST /api/v1/cache/clear` - Clear inference cache

## Quick Start

### 1. Setup Dependencies

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Start PostgreSQL and Redis (using Docker)
docker-compose up -d postgres redis
```

### 2. Configure Environment

```bash
cp .env.example .env
# Edit .env with your database credentials and AI API key
```

### 3. Initialize Database

```bash
cargo run -- --init-db
```

### 4. Bootstrap with Example Data

```bash
cargo run -- --bootstrap
```

### 5. Start Server

```bash
cargo run
# Server starts on http://127.0.0.1:7000
```

## Testing the API

### Generate a Matrix
```bash
curl -X POST http://127.0.0.1:7000/api/v1/flux/matrix/generate \
  -H "Content-Type: application/json" \
  -d '{"subject": "Machine Learning", "use_ai_generation": true}'
```

### Reverse Inference (Seeds → Meanings)
```bash
curl -X POST http://127.0.0.1:7000/api/v1/inference/reverse \
  -H "Content-Type: application/json" \
  -d '{
    "seed_numbers": [888, 872],
    "subject_filter": "Machine Learning",
    "include_synonyms": true,
    "include_antonyms": true
  }'
```

### Check Health
```bash
curl http://127.0.0.1:7000/api/v1/health
```

## Core Components

### Flux Matrix Engine (`src/flux_matrix.rs`)
- Implements the core 1,2,4,8,7,5,1 pattern
- Sacred guides at positions 3, 6, 9
- Node connections and relationships
- Seed number to flux sequence conversion

### Inference Engine (`src/inference_engine.rs`)
- Bidirectional number ↔ word processing
- Semantic association matching
- Moral alignment calculation
- Confidence scoring and contextual relevance

### Spatial Database (`src/spatial_database.rs`)
- PostgreSQL schema for matrices and associations
- Learning adjustments storage for RL
- Semantic search and categorization
- Statistics and analytics

### Cache Manager (`src/cache.rs`)
- Redis-based caching for performance
- Matrix pattern storage
- AI-generated content caching
- Access statistics and optimization

### AI Integration (`src/ai_integration.rs`)
- Grok 4 API integration
- Fallback to deterministic generation
- Semantic enhancement capabilities
- Matrix validation and enrichment

## Data Flow

1. **Subject Input** → Matrix generation (AI or deterministic)
2. **Seed Numbers** → Flux sequence conversion
3. **Sequence Matching** → Node position identification
4. **Semantic Lookup** → Association retrieval
5. **Moral Analysis** → Constructive/Destructive classification
6. **JSON Output** → Structured inference results

## Development

### Build
```bash
cargo build --release
```

### Test
```bash
cargo test
```

### Docker Deployment
```bash
docker-compose up --build
```

## Configuration

Key environment variables:
- `DATABASE_URL` - PostgreSQL connection string
- `REDIS_URL` - Redis connection string  
- `AI_API_KEY` - Grok 4 API key
- `PORT` - Server port (default: 7000)
- `RUST_LOG` - Logging configuration

## Features Implemented

✅ 9-node Flux Matrix with sacred guides (3,6,9)  
✅ Bidirectional number ↔ word inference  
✅ Semantic indexing with +/- moral alignment  
✅ PostgreSQL spatial database  
✅ Redis caching system  
✅ Grok 4 AI integration with fallback  
✅ REST API with comprehensive endpoints  
✅ Bootstrapping with example matrices  
✅ RL-ready learning adjustment tracking  
✅ JSON output for all inference results  

## Next Steps

1. **Frontend Interface** - Web UI for matrix visualization
2. **Vector Search** - Integration with embedding models
3. **Advanced RL** - Implement learning algorithm improvements
4. **Performance Optimization** - Query optimization and indexing
5. **API Documentation** - OpenAPI/Swagger specification
