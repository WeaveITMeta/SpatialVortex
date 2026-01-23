# EustressEngine ↔ SpatialVortex Integration Guide

This document describes how to integrate SpatialVortex's Universal Pipeline with EustressEngine's Parameters system for seamless data flow from any source to intelligent 3D visualization.

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         EUSTRESS ENGINE                                  │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │  Parameters System (crates/common/src/parameters.rs)            │   │
│  │  - DataSourceType (50+ connectors)                              │   │
│  │  - AuthType (OAuth2, AWS, Azure, etc.)                          │   │
│  │  - AnonymizationMode (Hash, Redact, DifferentialPrivacy)        │   │
│  │  - FieldMapping (source → target property mapping)              │   │
│  │  - Scopes (ExplicitScope, QueryScope, TagScope)                 │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                              ↓                                          │
│                    [WebTransport / REST API]                            │
└─────────────────────────────────────────────────────────────────────────┘
                               ↓
┌─────────────────────────────────────────────────────────────────────────┐
│                        SPATIAL VORTEX                                    │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │  Universal Pipeline (src/pipeline/)                             │   │
│  │  ┌─────────────┐                                                │   │
│  │  │ Input Layer │ ← Accept any data source                       │   │
│  │  └──────┬──────┘                                                │   │
│  │         ↓                                                        │   │
│  │  ┌─────────────────┐                                            │   │
│  │  │ Inference Layer │ ← Route, map fields, detect modality       │   │
│  │  └────────┬────────┘                                            │   │
│  │           ↓                                                      │   │
│  │  ┌──────────────────┐                                           │   │
│  │  │ Processing Layer │ ← Transform, embed, vectorize             │   │
│  │  └─────────┬────────┘                                           │   │
│  │            ↓                                                     │   │
│  │  ┌─────────────────────┐                                        │   │
│  │  │ Intelligence Layer  │ ← VortexModel reasoning, 3-6-9         │   │
│  │  └──────────┬──────────┘                                        │   │
│  │             ↓                                                    │   │
│  │  ┌──────────────┐                                               │   │
│  │  │ Output Layer │ → Multi-modal generation                      │   │
│  │  └──────────────┘                                               │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                              ↓                                          │
│                    [3D Visualization / API Response]                    │
└─────────────────────────────────────────────────────────────────────────┘
```

## Type Mapping: EustressEngine → SpatialVortex

### DataSourceType Mapping

| EustressEngine | SpatialVortex | Category |
|----------------|---------------|----------|
| `DataSourceType::REST` | `DataSourceType::REST` | api |
| `DataSourceType::GraphQL` | `DataSourceType::GraphQL` | api |
| `DataSourceType::FHIR` | `DataSourceType::FHIR` | healthcare |
| `DataSourceType::HL7v2` | `DataSourceType::HL7v2` | healthcare |
| `DataSourceType::MQTT` | `DataSourceType::MQTT` | streaming |
| `DataSourceType::Kafka` | `DataSourceType::Kafka` | streaming |
| `DataSourceType::WebSocket` | `DataSourceType::WebSocket` | streaming |
| `DataSourceType::WebTransport` | `DataSourceType::WebTransport` | streaming |
| `DataSourceType::PostgreSQL` | `DataSourceType::PostgreSQL` | database |
| `DataSourceType::S3` | `DataSourceType::S3` | cloud |
| `DataSourceType::OPCUA` | `DataSourceType::OPCUA` | iot |
| ... | ... | ... |

### AuthType Mapping

| EustressEngine | SpatialVortex |
|----------------|---------------|
| `AuthType::None` | `AuthType::None` |
| `AuthType::Basic` | `AuthType::Basic` |
| `AuthType::Bearer` | `AuthType::Bearer` |
| `AuthType::APIKey` | `AuthType::APIKey` |
| `AuthType::OAuth2` | `AuthType::OAuth2` |
| `AuthType::AWSSignature` | `AuthType::AWSSignature` |
| `AuthType::AzureAD` | `AuthType::AzureAD` |
| `AuthType::GoogleServiceAccount` | `AuthType::GoogleServiceAccount` |

### AnonymizationMode Mapping

| EustressEngine | SpatialVortex |
|----------------|---------------|
| `AnonymizationMode::None` | `AnonymizationMode::None` |
| `AnonymizationMode::Hash` | `AnonymizationMode::Hash` |
| `AnonymizationMode::Synthetic` | `AnonymizationMode::Synthetic` |
| `AnonymizationMode::Redact` | `AnonymizationMode::Redact` |
| `AnonymizationMode::KAnonymity` | `AnonymizationMode::KAnonymity` |
| `AnonymizationMode::DifferentialPrivacy` | `AnonymizationMode::DifferentialPrivacy` |

## Integration Code

### Converting EustressEngine Parameters to SpatialVortex RawInput

```rust
// In EustressEngine: Convert Parameters to SpatialVortex RawInput
use spatial_vortex::pipeline::{RawInput, RawContent, DataSourceType, AuthType, AnonymizationMode};

impl From<&eustress_common::Parameters> for RawInput {
    fn from(params: &eustress_common::Parameters) -> Self {
        RawInput {
            source_type: convert_source_type(params.data_source_type),
            content: RawContent::Text(String::new()), // Will be filled with actual data
            endpoint: params.endpoint.clone(),
            resource_id: params.resource_id.clone(),
            content_type: None,
            metadata: params.custom_params.clone(),
            auth: convert_auth_type(&params.auth_type),
            anonymization: convert_anonymization(params.anonymization_mode),
        }
    }
}

fn convert_source_type(src: eustress_common::DataSourceType) -> DataSourceType {
    match src {
        eustress_common::DataSourceType::REST => DataSourceType::REST,
        eustress_common::DataSourceType::GraphQL => DataSourceType::GraphQL,
        eustress_common::DataSourceType::FHIR => DataSourceType::FHIR,
        eustress_common::DataSourceType::MQTT => DataSourceType::MQTT,
        eustress_common::DataSourceType::Kafka => DataSourceType::Kafka,
        eustress_common::DataSourceType::WebSocket => DataSourceType::WebSocket,
        eustress_common::DataSourceType::WebTransport => DataSourceType::WebTransport,
        eustress_common::DataSourceType::PostgreSQL => DataSourceType::PostgreSQL,
        eustress_common::DataSourceType::S3 => DataSourceType::S3,
        // ... map all other types
        _ => DataSourceType::None,
    }
}

fn convert_auth_type(auth: &eustress_common::AuthType) -> AuthType {
    match auth {
        eustress_common::AuthType::None => AuthType::None,
        eustress_common::AuthType::Basic => AuthType::Basic,
        eustress_common::AuthType::Bearer => AuthType::Bearer,
        eustress_common::AuthType::APIKey => AuthType::APIKey,
        eustress_common::AuthType::OAuth2 => AuthType::OAuth2,
        eustress_common::AuthType::AWSSignature => AuthType::AWSSignature,
        eustress_common::AuthType::AzureAD => AuthType::AzureAD,
        eustress_common::AuthType::GoogleServiceAccount => AuthType::GoogleServiceAccount,
        _ => AuthType::None,
    }
}

fn convert_anonymization(mode: eustress_common::AnonymizationMode) -> AnonymizationMode {
    match mode {
        eustress_common::AnonymizationMode::None => AnonymizationMode::None,
        eustress_common::AnonymizationMode::Hash => AnonymizationMode::Hash,
        eustress_common::AnonymizationMode::Synthetic => AnonymizationMode::Synthetic,
        eustress_common::AnonymizationMode::Redact => AnonymizationMode::Redact,
        eustress_common::AnonymizationMode::KAnonymity => AnonymizationMode::KAnonymity,
        eustress_common::AnonymizationMode::DifferentialPrivacy => AnonymizationMode::DifferentialPrivacy,
    }
}
```

### Converting FieldMapping

```rust
// Convert EustressEngine FieldMapping to SpatialVortex FieldMapping
use spatial_vortex::pipeline::inference_layer::{FieldMapping, MappingTargetType};

impl From<&eustress_common::FieldMapping> for FieldMapping {
    fn from(mapping: &eustress_common::FieldMapping) -> Self {
        FieldMapping {
            source_path: mapping.source_path.clone(),
            target_property: mapping.target_property.clone(),
            target_type: convert_target_type(mapping.target_type),
            transform: mapping.transform.clone(),
            default_value: mapping.default_value.clone(),
            required: mapping.required,
        }
    }
}

fn convert_target_type(target: eustress_common::MappingTargetType) -> MappingTargetType {
    match target {
        eustress_common::MappingTargetType::Attribute => MappingTargetType::Attribute,
        eustress_common::MappingTargetType::Color => MappingTargetType::Color,
        eustress_common::MappingTargetType::Position => MappingTargetType::Position,
        eustress_common::MappingTargetType::Size => MappingTargetType::Size,
        eustress_common::MappingTargetType::Rotation => MappingTargetType::Rotation,
        eustress_common::MappingTargetType::Name => MappingTargetType::Name,
        eustress_common::MappingTargetType::Visible => MappingTargetType::Visible,
        _ => MappingTargetType::Attribute,
    }
}
```

## Processing Flow Example

### Healthcare Data (FHIR) → 3D Visualization

```rust
use spatial_vortex::pipeline::{UniversalPipeline, RawInput, RawContent, DataSourceType};

async fn process_fhir_patient(patient_json: serde_json::Value) -> Result<OutputResult, PipelineError> {
    // Create pipeline
    let pipeline = UniversalPipeline::new();
    
    // Create input from FHIR data
    let input = RawInput::from_json(patient_json)
        .with_source_type(DataSourceType::FHIR)
        .with_endpoint("https://fhir.example.com/Patient")
        .with_metadata("domain", "healthcare")
        .with_metadata("schema_version", "R4");
    
    // Process through all 5 layers
    let result = pipeline.process(input).await?;
    
    // Result contains:
    // - Insights about patient data
    // - Hypotheses for clinical decision support
    // - ELP tensor (Ethos-Logos-Pathos balance)
    // - Signal strength for confidence
    // - Structured output for 3D visualization
    
    Ok(result)
}
```

### IoT Sensor Data → Real-time Analysis

```rust
use spatial_vortex::pipeline::{UniversalPipeline, RawInput, RawContent, DataSourceType};

async fn process_sensor_stream(sensor_data: &[u8]) -> Result<OutputResult, PipelineError> {
    let pipeline = UniversalPipeline::new();
    
    let input = RawInput::from_bytes(sensor_data.to_vec())
        .with_source_type(DataSourceType::MQTT)
        .with_endpoint("mqtt://sensors.example.com/temperature")
        .with_metadata("domain", "iot")
        .with_metadata("device_id", "sensor-001");
    
    pipeline.process(input).await
}
```

## Domain Configuration

### Registering Healthcare Domain

```rust
use spatial_vortex::pipeline::inference_layer::{InferenceLayer, DomainConfig, FieldMapping};

fn configure_healthcare_domain(inference: &InferenceLayer) {
    let healthcare = DomainConfig {
        name: "healthcare".to_string(),
        description: "Healthcare and medical data processing".to_string(),
        default_mappings: vec![
            FieldMapping::new("patient.name.given[0]", "display_name"),
            FieldMapping::new("patient.birthDate", "birth_date"),
            FieldMapping::new("observation.valueQuantity.value", "vital_value"),
        ],
        pipelines: vec!["text".to_string(), "structured".to_string()],
        keywords: vec![
            "patient".into(), "fhir".into(), "hl7".into(), 
            "diagnosis".into(), "observation".into(), "medication".into()
        ],
        elp_bias: [0.5, 0.3, 0.2], // Ethics-heavy for healthcare
    };
    
    inference.register_domain(healthcare);
}
```

## Sacred Geometry Integration

SpatialVortex applies sacred geometry checkpoints (3-6-9) during intelligence processing:

```
Reasoning Chain:
  Step 1: Feature Analysis
  Step 2: Embedding Analysis
  Step 3: Sacred Checkpoint (Position 3 - Ethos) ← Intervention if signal weak
  Step 4: Domain Reasoning
  Step 5: Pattern Matching
  Step 6: Sacred Checkpoint (Position 6 - Logos) ← Intervention if signal weak
  Step 7: Hypothesis Generation
  Step 8: Knowledge Extraction
  Step 9: Sacred Checkpoint (Position 9 - Pathos) ← Final validation
```

### VCP (Vortex Context Preserver) Thresholds

| Confidence | Action |
|-----------------|--------|
| ≥ 0.7 | No intervention needed |
| 0.5 - 0.7 | Minor boost at sacred positions |
| 0.3 - 0.5 | Significant intervention |
| < 0.3 | High hallucination risk, major correction |

## ELP Tensor for 3D Visualization

The ELP (Ethos-Logos-Pathos) tensor can drive 3D visualization properties:

```rust
// Map ELP to visual properties
fn elp_to_visual(elp: [f32; 3]) -> VisualProperties {
    VisualProperties {
        // Ethos → Blue channel (trust, ethics)
        color_b: elp[0],
        // Logos → Green channel (logic, reason)
        color_g: elp[1],
        // Pathos → Red channel (emotion, feeling)
        color_r: elp[2],
        // Signal strength → Opacity
        opacity: confidence,
        // Confidence → Size
        scale: 0.5 + confidence * 0.5,
    }
}
```

## File Locations

### SpatialVortex Pipeline
- `src/pipeline/mod.rs` - Main pipeline orchestrator
- `src/pipeline/input_layer.rs` - Input data layer
- `src/pipeline/inference_layer.rs` - Routing and context
- `src/pipeline/processing_layer.rs` - Modality transforms
- `src/pipeline/intelligence_layer.rs` - VortexModel reasoning
- `src/pipeline/output_layer.rs` - Multi-modal generation
- `src/pipeline/data_types.rs` - Universal data types

### EustressEngine Parameters
- `crates/common/src/parameters.rs` - Parameters system

## Next Steps for Full Integration

1. **Create shared crate**: `spatial-llm` with type conversions E:\Workspace\EustressEngine\eustress\crates\spatial-llm
2. **WebTransport bridge**: Real-time data streaming between engines - FOUNDATIONS DONE
3. **EustressEngine integration**: 3D visualization of pipeline outputs
4. **Scopes integration**: Map EustressEngine scopes to SpatialVortex routing rules
5. **Validation rules**: Integrate EustressEngine ValidationRule with pipeline validation

## Dependencies to Add

### In EustressEngine's Cargo.toml
```toml
[dependencies]
spatial-vortex = { path = "../SpatialVortex" }
```

### In SpatialVortex's Cargo.toml (optional)
```toml
[dependencies]
eustress-common = { path = "../EustressEngine/eustress/crates/common", optional = true }

[features]
eustress = ["eustress-common"]
```
