# Swagger Updates - Priority Implementation Guide

**Date**: October 29, 2025  
**Purpose**: Step-by-step guide to update Swagger for production readiness

---

## Immediate Actions (This Week)

### 1. Add Error Response Standards (2 hours)

Add to `api/swagger.yml`:

```yaml
components:
  schemas:
    ErrorResponse:
      type: object
      required:
        - error
        - message
        - timestamp
      properties:
        error:
          type: string
          enum:
            - bad_request
            - unauthorized
            - forbidden
            - not_found
            - rate_limit_exceeded
            - internal_error
            - service_unavailable
            - validation_error
        message:
          type: string
        details:
          type: object
        timestamp:
          type: string
          format: date-time
        trace_id:
          type: string
        path:
          type: string

  responses:
    BadRequest:
      description: Bad Request
      content:
        application/json:
          schema:
            $ref: '#/components/schemas/ErrorResponse'
    Unauthorized:
      description: Unauthorized - Missing or invalid authentication
      content:
        application/json:
          schema:
            $ref: '#/components/schemas/ErrorResponse'
    Forbidden:
      description: Forbidden - Insufficient permissions
      content:
        application/json:
          schema:
            $ref: '#/components/schemas/ErrorResponse'
    NotFound:
      description: Resource not found
      content:
        application/json:
          schema:
            $ref: '#/components/schemas/ErrorResponse'
    RateLimitExceeded:
      description: Rate limit exceeded
      headers:
        X-RateLimit-Limit:
          schema:
            type: integer
        X-RateLimit-Remaining:
          schema:
            type: integer
        X-RateLimit-Reset:
          schema:
            type: integer
      content:
        application/json:
          schema:
            $ref: '#/components/schemas/ErrorResponse'
    InternalError:
      description: Internal server error
      content:
        application/json:
          schema:
            $ref: '#/components/schemas/ErrorResponse'
```

---

### 2. Document Existing Chat API (1 hour)

Add to `paths:` section:

```yaml
  /chat/text:
    post:
      summary: Chat with AI using ONNX embeddings
      description: |
        Process natural language chat with:
        - ONNX sentence-transformers embeddings
        - Sacred geometry (3-6-9) transformation
        - ELP channel analysis
        - Flux position calculation
      tags:
        - Chat & Conversation
      security:
        - BearerAuth: []
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/ChatRequest'
      responses:
        '200':
          description: Chat response generated successfully
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/ChatResponse'
        '400':
          $ref: '#/components/responses/BadRequest'
        '401':
          $ref: '#/components/responses/Unauthorized'
        '429':
          $ref: '#/components/responses/RateLimitExceeded'
        '500':
          $ref: '#/components/responses/InternalError'
```

Add schemas:

```yaml
  ChatRequest:
    type: object
    required:
      - message
      - user_id
    properties:
      message:
        type: string
        description: User message
        example: "What is consciousness?"
      user_id:
        type: string
        description: Unique user identifier
      context:
        type: array
        items:
          type: string
        description: Optional conversation context

  ChatResponse:
    type: object
    properties:
      response:
        type: string
        description: AI generated response
      elp_values:
        $ref: '#/components/schemas/ELPValues'
      confidence:
        type: number
        format: double
        minimum: 0
        maximum: 1
      flux_position:
        type: integer
        minimum: 0
        maximum: 9
      confidence:
        type: number
        format: double
        minimum: 0
        maximum: 1
      processing_time_ms:
        type: integer
      subject:
        type: string

  ELPValues:
    type: object
    properties:
      ethos:
        type: number
        format: float
        description: Character/moral dimension (0-13)
      logos:
        type: number
        format: float
        description: Logic/reason dimension (0-13)
      pathos:
        type: number
        format: float
        description: Emotion/feeling dimension (0-13)
```

---

### 3. Add Security Schemes (30 minutes)

Update `components/securitySchemes`:

```yaml
components:
  securitySchemes:
    BearerAuth:
      type: http
      scheme: bearer
      bearerFormat: JWT
      description: |
        JWT token obtained from /auth/login endpoint.
        Include in Authorization header: `Bearer <token>`
    ApiKeyAuth:
      type: apiKey
      in: header
      name: X-API-Key
      description: |
        API key obtained from /auth/api-keys endpoint.
        Include in X-API-Key header

# Apply globally (or per-endpoint)
security:
  - BearerAuth: []
  - ApiKeyAuth: []
```

---

### 4. Document ASI Orchestrator Endpoints (30 minutes)

These already exist in `src/ai/endpoints.rs`:

```yaml
  /ml/embed:
    post:
      summary: Generate ONNX embeddings
      description: Generate embeddings with optional sacred geometry transformation
      tags:
        - Machine Learning
      security:
        - BearerAuth: []
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/OnnxEmbedRequest'
      responses:
        '200':
          description: Embedding generated successfully
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/OnnxEmbedResponse'
        '503':
          description: ONNX feature not enabled

  /ml/asi/infer:
    post:
      summary: ASI inference with sacred geometry
      description: Full sacred geometry pipeline with ELP analysis
      tags:
        - Machine Learning
      security:
        - BearerAuth: []
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/ASIInferenceRequest'
      responses:
        '200':
          description: ASI inference completed
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/ASIInferenceResponse'

  /ml/asi/metrics:
    get:
      summary: Get ASI performance metrics
      description: Retrieve performance metrics for ASI orchestrator
      tags:
        - Machine Learning
      security:
        - BearerAuth: []
      responses:
        '200':
          description: Metrics retrieved successfully
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/ASIMetrics'

  /ml/asi/weights:
    get:
      summary: Get ASI adaptive weights
      description: Retrieve current adaptive weights for geometric/ML/consensus
      tags:
        - Machine Learning
      security:
        - BearerAuth: []
      responses:
        '200':
          description: Weights retrieved successfully
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/ASIWeights'
```

---

### 5. Document Storage Endpoints (15 minutes)

```yaml
  /storage/confidence-lake/status:
    get:
      summary: Confidence Lake status
      description: Get status and statistics of Confidence Lake
      tags:
        - Storage
      security:
        - BearerAuth: []
      responses:
        '200':
          description: Status retrieved successfully
          content:
            application/json:
              schema:
                type: object
                properties:
                  status:
                    type: string
                    enum: [ready, degraded, unavailable]
                  encryption_enabled:
                    type: boolean
                  total_entries:
                    type: integer
                  signal_threshold:
                    type: number
                    format: float
                  used_space_mb:
                    type: number
                  available_space_mb:
                    type: number
        '503':
          description: Confidence Lake feature not enabled

  /voice/status:
    get:
      summary: Voice pipeline status
      description: Get status of voice processing pipeline
      tags:
        - Voice Pipeline
      security:
        - BearerAuth: []
      responses:
        '200':
          description: Status retrieved successfully
          content:
            application/json:
              schema:
                type: object
                properties:
                  status:
                    type: string
                    enum: [ready, degraded, unavailable]
                  sample_rate:
                    type: integer
                  fft_enabled:
                    type: boolean
                  elp_mapping_enabled:
                    type: boolean
        '503':
          description: Voice feature not enabled
```

---

## Week 1 Priority: Authentication APIs

### Authentication Endpoints to Add

```yaml
  /auth/register:
    post:
      summary: Register new user
      description: Create new user account
      tags:
        - Authentication
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
              required:
                - username
                - email
                - password
              properties:
                username:
                  type: string
                  minLength: 3
                  maxLength: 32
                email:
                  type: string
                  format: email
                password:
                  type: string
                  format: password
                  minLength: 8
                organization:
                  type: string
      responses:
        '201':
          description: User registered successfully
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/AuthResponse'
        '400':
          $ref: '#/components/responses/BadRequest'
        '409':
          description: User already exists

  /auth/login:
    post:
      summary: User login
      description: Authenticate and receive JWT token
      tags:
        - Authentication
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
              required:
                - email
                - password
              properties:
                email:
                  type: string
                  format: email
                password:
                  type: string
                  format: password
      responses:
        '200':
          description: Login successful
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/AuthResponse'
        '401':
          $ref: '#/components/responses/Unauthorized'

  /auth/refresh:
    post:
      summary: Refresh JWT token
      description: Get new access token using refresh token
      tags:
        - Authentication
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
              required:
                - refresh_token
              properties:
                refresh_token:
                  type: string
      responses:
        '200':
          description: Token refreshed successfully
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/AuthResponse'
        '401':
          $ref: '#/components/responses/Unauthorized'

  /auth/logout:
    post:
      summary: Logout user
      description: Invalidate current JWT token
      tags:
        - Authentication
      security:
        - BearerAuth: []
      responses:
        '200':
          description: Logout successful
        '401':
          $ref: '#/components/responses/Unauthorized'

  /auth/me:
    get:
      summary: Get current user
      description: Retrieve authenticated user information
      tags:
        - Authentication
      security:
        - BearerAuth: []
      responses:
        '200':
          description: User information retrieved
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/User'
        '401':
          $ref: '#/components/responses/Unauthorized'

  /auth/api-keys:
    get:
      summary: List API keys
      description: List all API keys for current user
      tags:
        - Authentication
      security:
        - BearerAuth: []
      responses:
        '200':
          description: API keys retrieved
          content:
            application/json:
              schema:
                type: array
                items:
                  $ref: '#/components/schemas/APIKey'
    post:
      summary: Create API key
      description: Generate new API key
      tags:
        - Authentication
      security:
        - BearerAuth: []
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
              required:
                - name
              properties:
                name:
                  type: string
                expires_in_days:
                  type: integer
                  default: 90
                permissions:
                  type: array
                  items:
                    type: string
      responses:
        '201':
          description: API key created
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/APIKey'

  /auth/api-keys/{id}:
    delete:
      summary: Revoke API key
      description: Delete/revoke an API key
      tags:
        - Authentication
      security:
        - BearerAuth: []
      parameters:
        - name: id
          in: path
          required: true
          schema:
            type: string
      responses:
        '204':
          description: API key revoked
        '404':
          $ref: '#/components/responses/NotFound'
```

**Schemas**:

```yaml
components:
  schemas:
    AuthResponse:
      type: object
      properties:
        token:
          type: string
          description: JWT access token
        refresh_token:
          type: string
          description: Refresh token for getting new access token
        expires_in:
          type: integer
          description: Token expiration in seconds
        user:
          $ref: '#/components/schemas/User'

    User:
      type: object
      properties:
        id:
          type: string
          format: uuid
        email:
          type: string
          format: email
        username:
          type: string
        role:
          type: string
          enum: [user, admin, superadmin]
        organization:
          type: string
        created_at:
          type: string
          format: date-time
        last_login:
          type: string
          format: date-time

    APIKey:
      type: object
      properties:
        id:
          type: string
          format: uuid
        key:
          type: string
          description: Only returned on creation
        name:
          type: string
        created_at:
          type: string
          format: date-time
        expires_at:
          type: string
          format: date-time
        last_used:
          type: string
          format: date-time
        permissions:
          type: array
          items:
            type: string
```

---

## Week 2-3: RAG System APIs

### Core RAG Endpoints Structure

```yaml
paths:
  # Document Ingestion
  /rag/ingest/file:
    post:
      summary: Ingest document from file
      tags: [RAG - Document Management]
  
  /rag/ingest/directory:
    post:
      summary: Ingest entire directory
      tags: [RAG - Document Management]
  
  /rag/ingest/url:
    post:
      summary: Ingest document from URL
      tags: [RAG - Document Management]
  
  /rag/documents:
    get:
      summary: List ingested documents
      tags: [RAG - Document Management]
  
  # Vector Search
  /rag/search:
    post:
      summary: Semantic search
      tags: [RAG - Search & Retrieval]
  
  /rag/retrieve:
    post:
      summary: Retrieve context for query
      tags: [RAG - Search & Retrieval]
  
  /rag/retrieve/sacred:
    post:
      summary: Sacred geometry filtered retrieval
      tags: [RAG - Search & Retrieval]
  
  # Generation
  /rag/generate:
    post:
      summary: Generate with retrieved context
      tags: [RAG - Generation]
  
  /rag/generate/stream:
    post:
      summary: Streaming generation
      tags: [RAG - Generation]
  
  # Training
  /rag/training/start:
    post:
      summary: Start continuous learning
      tags: [RAG - Training]
  
  /rag/training/metrics:
    get:
      summary: Get learning metrics
      tags: [RAG - Training]
```

---

## Recommended Tag Structure

Update `tags:` section in Swagger:

```yaml
tags:
  - name: System
    description: Health checks and system status
  - name: Authentication
    description: User authentication and API key management
  - name: Chat & Conversation
    description: Conversational AI with sacred geometry
  - name: Flux Matrix
    description: Flux matrix generation and manipulation
  - name: Flux Nodes
    description: Individual flux node operations
  - name: Sacred Geometry
    description: Sacred geometry calculations (3-6-9 patterns)
  - name: AI Inference
    description: Semantic inference and processing
  - name: Machine Learning
    description: ML training, inference, and models
  - name: RAG - Document Management
    description: Document ingestion and management
  - name: RAG - Search & Retrieval
    description: Vector search and context retrieval
  - name: RAG - Generation
    description: Augmented generation with RAG
  - name: RAG - Training
    description: Continuous learning and training
  - name: Storage
    description: Confidence Lake and data persistence
  - name: Voice Pipeline
    description: Voice capture and processing
  - name: Coding Agent
    description: Multi-language code generation and execution
  - name: Monitoring
    description: Metrics, logs, and observability
  - name: Batch Processing
    description: Batch operations for high throughput
  - name: Visualization
    description: 2D/3D visualizations and exports
  - name: Admin
    description: Administrative operations
```

---

## Common Parameters to Add

```yaml
components:
  parameters:
    PageParam:
      name: page
      in: query
      description: Page number for pagination
      schema:
        type: integer
        minimum: 1
        default: 1
    
    LimitParam:
      name: limit
      in: query
      description: Number of results per page
      schema:
        type: integer
        minimum: 1
        maximum: 100
        default: 20
    
    SortParam:
      name: sort
      in: query
      description: Sort direction
      schema:
        type: string
        enum: [asc, desc]
        default: desc
    
    FilterParam:
      name: filter
      in: query
      description: JSON filter expression
      schema:
        type: string
```

---

## Response Headers to Add

```yaml
components:
  headers:
    X-Request-ID:
      description: Unique request identifier for tracing
      schema:
        type: string
        format: uuid
    
    X-RateLimit-Limit:
      description: Maximum requests per window
      schema:
        type: integer
    
    X-RateLimit-Remaining:
      description: Requests remaining in current window
      schema:
        type: integer
    
    X-RateLimit-Reset:
      description: Unix timestamp when rate limit resets
      schema:
        type: integer
        format: int64
    
    X-Processing-Time:
      description: Server processing time in milliseconds
      schema:
        type: integer
```

---

## Implementation Checklist

### Immediate (Today)
- [ ] Add error response schemas
- [ ] Add security schemes
- [ ] Document existing `/chat/text` endpoint
- [ ] Document existing ASI endpoints (4 endpoints)
- [ ] Document existing storage endpoints (2 endpoints)
- [ ] Add common parameters
- [ ] Add response headers
- [ ] Update tag structure

### Week 1
- [ ] Implement authentication backend (JWT, bcrypt)
- [ ] Add all 8 authentication endpoints to Swagger
- [ ] Test authentication flow
- [ ] Add rate limiting configuration

### Week 2-3
- [ ] Create RAG API wrappers (15 endpoints)
- [ ] Document all RAG endpoints in Swagger
- [ ] Test RAG workflows
- [ ] Add monitoring endpoints (9 endpoints)

### Week 4-5
- [ ] ML training API wrappers (15 endpoints)
- [ ] Confidence Lake query APIs (9 endpoints)
- [ ] Voice pipeline APIs (8 endpoints)
- [ ] Coding agent APIs (8 endpoints)

---

## Testing Strategy

### Swagger Validation
```bash
# Install swagger validator
npm install -g @apidevtools/swagger-cli

# Validate swagger file
swagger-cli validate api/swagger.yml
```

### API Testing with Swagger UI
1. Deploy Swagger UI with updated spec
2. Test each endpoint interactively
3. Verify request/response schemas
4. Check error handling

### Automated Testing
```bash
# Generate API client from Swagger
openapi-generator-cli generate -i api/swagger.yml -g rust -o api-client/

# Run integration tests
cargo test --features api-tests
```

---

## Documentation Best Practices

1. **Every endpoint must have**:
   - Clear summary (< 50 chars)
   - Detailed description
   - Request/response examples
   - All possible status codes
   - Security requirements
   - Rate limit information

2. **All schemas must have**:
   - Property descriptions
   - Type and format
   - Validation rules (min, max, pattern)
   - Required vs optional fields
   - Examples

3. **Use $ref extensively**:
   - Reuse common schemas
   - Reference common responses
   - Share parameters across endpoints

4. **Keep schemas DRY**:
   - Extract common patterns
   - Use composition (allOf, oneOf)
   - Create base schemas for inheritance

---

## Next Steps

1. **Review this document** with team
2. **Approve immediate actions** (today's work)
3. **Assign Week 1 tasks** to developers
4. **Set up CI/CD** for Swagger validation
5. **Deploy Swagger UI** to staging
6. **Begin implementation** following this guide

---

**Estimated Total Effort**: 
- Immediate updates: 4 hours
- Week 1: 40 hours (1 developer week)
- Weeks 2-5: 160 hours (4 developer weeks)
- Total: ~200 hours (5 developer weeks)
