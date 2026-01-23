# Visual Subject Generation Application Programming Interface

## Overview

The Visual Subject Generation Application Programming Interface (API) analyzes 2D flux matrix visualizations and dynamically generates subject definitions based on spatial patterns, sacred geometry intersections, and Ethos-Logos-Pathos (ELP) channel dominance.

## Architecture

### Pipeline Flow

```
2D Flux Matrix Image
  → Extract Visual Data (positions, colors, intersections)
  → Analyze Patterns (ELP dominance, sacred clusters, flow intensity)
  → Generate Artificial Intelligence (AI) Prompt
  → Parse AI Response
  → Create Subject Module Files
  → Update subjects/mod.rs
```

### Key Components

1. **Visual Data Extraction**: Converts rendered flux matrix into structured data
2. **Pattern Analysis**: Identifies dominant channels and sacred intersections
3. **AI-Powered Generation**: Uses visual insights to create meaningful subjects
4. **File System Integration**: Automatically creates and registers new modules

## API Endpoints

### 1. Generate Subject from Visual Data

**Endpoint**: `POST /api/v1/subjects/generate-from-visual`

**Description**: Creates a new subject definition by analyzing visual flux matrix data extracted from 2D renderings.

**Request Body**:
```json
{
  "subject_name": "Consciousness",
  "visual_data": {
    "subject": "Consciousness",
    "node_positions": {
      "1": {
        "position": 1,
        "coordinates": [2.0, 2.0],
        "color": {
          "r": 0.2,
          "g": 0.8,
          "b": 0.6
        },
        "scale": 1.2,
        "connections": [2, 4],
        "is_sacred": false
      },
      "3": {
        "position": 3,
        "coordinates": [4.0, -2.0],
        "color": {
          "r": 0.0,
          "g": 1.0,
          "b": 0.5
        },
        "scale": 1.5,
        "connections": [6, 9],
        "is_sacred": true
      }
    },
    "sacred_intersections": [
      {
        "coordinates": [0.0, 0.0],
        "sacred_positions": [3, 6, 9],
        "significance": 0.95
      }
    ],
    "flow_patterns": [
      {
        "from": 1,
        "to": 2,
        "curvature": 0.3,
        "intensity": 0.8
      }
    ]
  },
  "subjects_dir": "src/subjects"
}
```

**Response**:
```json
{
  "success": true,
  "subject_name": "Consciousness",
  "module_name": "consciousness",
  "filename": "consciousness.rs",
  "message": "Subject 'Consciousness' generated from visual analysis. Rebuild to use."
}
```

### 2. Analyze Existing Matrix Visual Data

**Endpoint**: `GET /api/v1/matrix/{subject}/visual-analysis`

**Description**: Extracts visual data from an existing flux matrix for analysis or debugging.

**Example**: `GET /api/v1/matrix/philosophy/visual-analysis`

**Response**:
```json
{
  "subject": "philosophy",
  "node_positions": { ... },
  "sacred_intersections": [ ... ],
  "flow_patterns": [ ... ]
}
```

### 3. Traditional Subject Generation (Text-Based)

**Endpoint**: `POST /api/v1/subjects/generate`

**Description**: Generates subject using AI prompts without visual analysis (legacy method).

**Request Body**:
```json
{
  "subject_name": "Ethics",
  "subjects_dir": "src/subjects"
}
```

## Visual Data Structure

### Node Visual Data

Each node in the 2D visualization contains:

| Field | Type | Description |
|-------|------|-------------|
| `position` | `u8` | Flux position (0-9) |
| `coordinates` | `(f32, f32)` | X,Y position in 2D space |
| `color` | `ColorData` | RGB values representing ELP channels |
| `scale` | `f32` | Size/importance indicator |
| `connections` | `Vec<u8>` | Connected node positions |
| `is_sacred` | `bool` | Sacred position (3, 6, 9) flag |

### Color Data (ELP Channels)

The RGB color values map to philosophical dimensions:

- **Red (r)**: **Pathos** - Emotion, feeling, passion
- **Green (g)**: **Logos** - Logic, reason, analysis  
- **Blue (b)**: **Ethos** - Character, ethics, morals

**Dominant Channel Analysis**:
```rust
// Determines which channel is strongest
if r > g && r > b { "pathos" }
else if g > r && g > b { "logos" }
else if b > r && b > g { "ethos" }
else { "balanced" }
```

### Sacred Intersections

Cyan dots in 2D visualization indicating key geometric intersections:

```json
{
  "coordinates": [x, y],
  "sacred_positions": [3, 6, 9],
  "significance": 0.0 - 1.0
}
```

**High significance** (>0.7) indicates critical organizing principles.

### Flow Patterns

Represents the doubling sequence **1 → 2 → 4 → 8 → 7 → 5 → 1**:

```json
{
  "from": 1,
  "to": 2,
  "curvature": 0.3,
  "intensity": 0.8
}
```

## AI Prompt Generation

The system creates rich prompts incorporating visual insights:

```
Generate a Spatial Vortex subject definition for: "Subject Name"

VISUAL ANALYSIS DATA FROM 2D FLUX MATRIX:
==========================================

1. COLOR DOMINANCE (ELP Channels):
   Position 1: logos dominant
   Position 2: pathos dominant
   Position 4: ethos dominant
   ...

2. SACRED INTERSECTIONS:
   - Total significant intersections: 5
   - High-significance clusters: [3, 9] (sig: 0.95)

3. FLOW PATTERNS (Doubling Sequence 1→2→4→8→7→5→1):
   1 → 2: intensity 0.80
   2 → 4: intensity 0.75
   ...

4. NODE POSITIONS & SCALE:
   Position 1: scale=1.20, sacred=false, connections=2
   Position 3: scale=1.50, sacred=true, connections=2
   ...
```

## Integration Examples

### Frontend Integration (TypeScript/JavaScript)

```typescript
// Extract visual data from canvas rendering
const visualData = {
  subject: "Consciousness",
  node_positions: extractNodesFromCanvas(),
  sacred_intersections: detectIntersections(),
  flow_patterns: analyzeFlowLines()
};

// Generate subject from visual data
const response = await fetch('/api/v1/subjects/generate-from-visual', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    subject_name: "Consciousness",
    visual_data: visualData
  })
});

const result = await response.json();
console.log(result.message);
// "Subject 'Consciousness' generated from visual analysis. Rebuild to use."
```

### Rust Integration

```rust
use spatial_vortex::visual_subject_generator::{
    VisualSubjectGenerator,
    FluxMatrixVisualData
};

// From existing matrix
let visual_data = VisualSubjectGenerator::extract_visual_data_from_matrix(&matrix);

// Generate subject
let visual_gen = VisualSubjectGenerator::new(ai_integration);
let subject = visual_gen.generate_from_visual_data(&visual_data).await?;
```

## Benefits Over Text-Based Generation

| Feature | Text-Based | Visual-Based |
|---------|------------|--------------|
| **Spatial Awareness** | ❌ None | ✅ Full 2D geometry |
| **ELP Channel Info** | ❌ Manual | ✅ Automatic from colors |
| **Sacred Geometry** | ❌ Ignored | ✅ Intersection analysis |
| **Flow Patterns** | ❌ Not considered | ✅ Intensity-aware |
| **Scale/Importance** | ❌ Equal weight | ✅ Visual size = importance |

## Error Handling

### Common Errors

**400 Bad Request**:
```json
{
  "success": false,
  "subject_name": "Invalid",
  "message": "Failed to analyze visual data: Missing color data"
}
```

**404 Not Found**:
```json
{
  "error": "Matrix not found for subject: nonexistent"
}
```

**500 Internal Server Error**:
```json
{
  "error": "AI service unavailable"
}
```

## Performance Considerations

- **Visual Analysis**: ~50ms per matrix
- **AI Generation**: ~2-5 seconds (depends on model)
- **File Writing**: ~10ms
- **Total Pipeline**: ~2-6 seconds per subject

## Security

- ✅ Input validation on all visual data fields
- ✅ File path sanitization (prevents directory traversal)
- ✅ Rate limiting on generation endpoints
- ✅ API key required for AI service calls

## Future Enhancements

1. **Direct Image Upload**: Accept PNG/JPEG flux matrix screenshots
2. **Batch Generation**: Process multiple subjects from single visualization
3. **Real-Time Updates**: WebSocket for live generation progress
4. **Visual Diff**: Compare generated subjects with existing ones
5. **3D Support**: Extend to analyze 3D Bevy renderings

## Related Documentation

- [Flux Matrix Core](./FLUX_MATRIX_CORE.md)
- [Sacred Geometry](./SACRED_GEOMETRY.md)
- [AI Integration](./AI_INTEGRATION.md)
- [Subject System](./SUBJECT_SYSTEM.md)
