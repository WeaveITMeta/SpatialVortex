# 🎨 Bevy 3D Visualization Shape Architecture

## Shape-Based Component System

### 📦 **Box (Cuboid) - Processing Blocks**

**Purpose**: Computational units, transformations, logic blocks

```rust
#[derive(Component)]
pub struct ProcessingBlock {
    /// Unique identifier
    pub id: String,
    
    /// Display label (centered text)
    pub label: String,
    
    /// Block type (transformation, inference, etc.)
    pub block_type: BlockType,
    
    /// Current processing state
    pub state: ProcessingState,
    
    /// Input connections
    pub inputs: Vec<NodeReference>,
    
    /// Output connections
    pub outputs: Vec<NodeReference>,
}

#[derive(Debug, Clone)]
pub enum BlockType {
    Transformation,      // Geometric → Flux conversion
    Inference,          // AI/ML inference block
    Aggregation,        // Data aggregation
    Routing,            // Data routing/switching
    Validation,         // Data validation
}

#[derive(Debug, Clone)]
pub enum ProcessingState {
    Idle,
    Processing,
    Complete,
    Error(String),
}
```

**Visual Properties**:
- Shape: Cuboid mesh
- Text: Centered label (entity name)
- Color: Based on state
  - Idle: Gray
  - Processing: Yellow
  - Complete: Green
  - Error: Red
- Size: Proportional to processing complexity

**Bevy Implementation**:
```rust
fn spawn_processing_block(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    block: &ProcessingBlock,
) -> Entity {
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(2.0, 1.0, 1.0)),
            material: materials.add(Color::srgba(0.5, 0.5, 0.5, 1.0)),
            transform: Transform::from_xyz(block.position.x, block.position.y, block.position.z),
            ..default()
        },
        block.clone(),
        Name::new(format!("Block: {}", block.label)),
    ))
    .with_children(|parent| {
        // Add text label
        parent.spawn(Text2dBundle {
            text: Text::from_section(
                &block.label,
                TextStyle {
                    font_size: 20.0,
                    color: Color::WHITE,
                    ..default()
                }
            ),
            transform: Transform::from_xyz(0.0, 0.0, 0.6),
            ..default()
        });
    })
    .id()
}
```

---

### 🗄️ **Cylinder - Database Connections**

**Purpose**: Real-time database access, data storage references

```rust
#[derive(Component)]
pub struct DatabaseNode {
    /// Unique identifier
    pub id: String,
    
    /// Database connection details
    pub connection: DatabaseConnection,
    
    /// Cached references
    pub references: Vec<DataReference>,
    
    /// Real-time access status
    pub status: ConnectionStatus,
    
    /// Query cache
    pub cache: QueryCache,
}

#[derive(Debug, Clone)]
pub struct DatabaseConnection {
    pub connection_string: String,
    pub db_type: DatabaseType,
    pub max_connections: usize,
    pub timeout: Duration,
}

#[derive(Debug, Clone)]
pub enum DatabaseType {
    PostgreSQL,
    Redis,
    MongoDB,
    Neo4j,        // Graph database
    SpatialDB,    // Custom spatial database
}

#[derive(Debug, Clone)]
pub struct DataReference {
    pub ref_id: String,
    pub table: String,
    pub key: String,
    pub cached_at: DateTime<Utc>,
    pub ttl: Duration,
}

#[derive(Debug, Clone)]
pub enum ConnectionStatus {
    Connected,
    Connecting,
    Disconnected,
    Error(String),
}

pub struct QueryCache {
    entries: HashMap<String, CachedQuery>,
    max_size: usize,
}

#[derive(Clone)]
struct CachedQuery {
    result: Vec<u8>,  // Serialized data
    timestamp: DateTime<Utc>,
    hits: u64,
}
```

**Visual Properties**:
- Shape: Cylinder mesh
- Orientation: Vertical (database tower)
- Color: Based on connection status
  - Connected: Blue
  - Connecting: Yellow
  - Disconnected: Gray
  - Error: Red
- Height: Proportional to data volume
- Radius: Fixed

**Bevy Implementation**:
```rust
fn spawn_database_node(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    db: &DatabaseNode,
) -> Entity {
    let color = match db.status {
        ConnectionStatus::Connected => Color::srgb(0.2, 0.4, 0.8),
        ConnectionStatus::Connecting => Color::srgb(0.8, 0.8, 0.2),
        ConnectionStatus::Disconnected => Color::srgb(0.5, 0.5, 0.5),
        ConnectionStatus::Error(_) => Color::srgb(0.8, 0.2, 0.2),
    };
    
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cylinder::new(0.5, 2.0)),
            material: materials.add(color),
            transform: Transform::from_xyz(db.position.x, db.position.y, db.position.z),
            ..default()
        },
        db.clone(),
        Name::new(format!("Database: {}", db.id)),
    ))
    .with_children(|parent| {
        // Add connection indicator rings
        for i in 0..3 {
            parent.spawn(PbrBundle {
                mesh: meshes.add(Torus::new(0.6, 0.1)),
                material: materials.add(Color::srgba(1.0, 1.0, 1.0, 0.3)),
                transform: Transform::from_xyz(0.0, 0.5 * i as f32, 0.0),
                ..default()
            });
        }
    })
    .id()
}
```

**Real-Time Data Processing**:
```rust
fn database_access_system(
    mut db_nodes: Query<&mut DatabaseNode>,
    mut commands: Commands,
) {
    for mut db in db_nodes.iter_mut() {
        // Process pending queries
        if db.status == ConnectionStatus::Connected {
            // Execute real-time queries
            for reference in &db.references {
                // Fetch data if cache expired
                if is_cache_expired(&reference) {
                    // Spawn async query task
                    let query = build_query(reference);
                    execute_database_query(&db.connection, query);
                }
            }
        }
    }
}
```

---

### ⚪ **Sphere - Node References**

**Purpose**: Metadata containers, reference nodes, information points

```rust
#[derive(Component)]
pub struct NodeReference {
    /// Unique identifier
    pub id: String,
    
    /// Node metadata
    pub metadata: NodeMetadata,
    
    /// Reference type
    pub ref_type: ReferenceType,
    
    /// Connected entities
    pub connections: Vec<EntityConnection>,
    
    /// Visibility state
    pub visible: bool,
}

#[derive(Debug, Clone)]
pub struct NodeMetadata {
    /// Display name
    pub name: String,
    
    /// Flux position (0-9)
    pub position: u8,
    
    /// ELP tensor
    pub elp: ELPTensor,
    
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    
    /// Last accessed
    pub last_accessed: DateTime<Utc>,
    
    /// Access count
    pub access_count: u64,
    
    /// Custom properties
    pub properties: HashMap<String, MetadataValue>,
    
    /// Tags for categorization
    pub tags: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum MetadataValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Array(Vec<String>),
    Tensor(ELPTensor),
}

#[derive(Debug, Clone)]
pub enum ReferenceType {
    FluxNode,           // References a flux matrix node
    ProcessingBlock,    // References a processing block
    DatabaseEntry,      // References database data
    ExternalAPI,        // References external API
    UserDefined,        // Custom reference
}

#[derive(Debug, Clone)]
pub struct EntityConnection {
    pub target_id: String,
    pub connection_type: ConnectionType,
    pub strength: f32,  // 0.0-1.0
    pub bidirectional: bool,
}

#[derive(Debug, Clone)]
pub enum ConnectionType {
    DataFlow,      // Data flows from A to B
    Reference,     // A references B
    Dependency,    // A depends on B
    Similarity,    // A is similar to B
}
```

**Visual Properties**:
- Shape: Sphere mesh (UV sphere)
- Color: Based on ELP tensor
  - Red channel: Ethos
  - Green channel: Logos
  - Blue channel: Pathos
- Size: Proportional to access count
- Glow: Indicates recent activity
- Orbit: Around connected processing blocks

**Bevy Implementation**:
```rust
fn spawn_node_reference(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    node: &NodeReference,
) -> Entity {
    // Color based on ELP tensor
    let color = Color::srgb(
        node.metadata.elp.ethos as f32,
        node.metadata.elp.logos as f32,
        node.metadata.elp.pathos as f32,
    );
    
    // Size based on access count
    let radius = 0.3 + (node.metadata.access_count as f32).log10().max(0.0) * 0.1;
    
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Sphere::new(radius)),
            material: materials.add(StandardMaterial {
                base_color: color,
                emissive: color * 0.5,  // Glow effect
                ..default()
            }),
            transform: Transform::from_xyz(
                node.position.x,
                node.position.y,
                node.position.z,
            ),
            ..default()
        },
        node.clone(),
        Name::new(format!("Node: {}", node.metadata.name)),
    ))
    .with_children(|parent| {
        // Add metadata indicator (small ring)
        parent.spawn(PbrBundle {
            mesh: meshes.add(Torus::new(radius * 1.2, 0.05)),
            material: materials.add(Color::srgba(1.0, 1.0, 1.0, 0.5)),
            transform: Transform::from_rotation(Quat::from_rotation_x(PI / 2.0)),
            ..default()
        });
    })
    .id()
}
```

**Metadata Access System**:
```rust
fn metadata_access_system(
    mut nodes: Query<&mut NodeReference>,
    time: Res<Time>,
) {
    for mut node in nodes.iter_mut() {
        // Update last accessed timestamp
        node.metadata.last_accessed = Utc::now();
        node.metadata.access_count += 1;
        
        // Process metadata queries
        for (key, value) in &node.metadata.properties {
            match value {
                MetadataValue::Tensor(tensor) => {
                    // Update visualization based on tensor
                    // This would trigger color/glow updates
                },
                _ => {}
            }
        }
    }
}
```

---

## 🔗 **Connection Visualization**

### Line Connections Between Shapes

```rust
#[derive(Component)]
pub struct Connection {
    pub from: Entity,
    pub to: Entity,
    pub connection_type: ConnectionType,
    pub data_flow: DataFlow,
    pub active: bool,
}

#[derive(Debug, Clone)]
pub struct DataFlow {
    pub bandwidth: f32,  // Data throughput
    pub latency: Duration,
    pub packets: Vec<DataPacket>,
}

#[derive(Debug, Clone)]
pub struct DataPacket {
    pub id: String,
    pub data: Vec<u8>,
    pub position: f32,  // 0.0-1.0 along connection line
}
```

**Visual Implementation**:
```rust
fn draw_connections(
    mut gizmos: Gizmos,
    connections: Query<&Connection>,
    transforms: Query<&Transform>,
) {
    for conn in connections.iter() {
        if let (Ok(from_transform), Ok(to_transform)) = 
            (transforms.get(conn.from), transforms.get(conn.to)) 
        {
            let from_pos = from_transform.translation;
            let to_pos = to_transform.translation;
            
            // Color based on connection type
            let color = match conn.connection_type {
                ConnectionType::DataFlow => Color::GREEN,
                ConnectionType::Reference => Color::BLUE,
                ConnectionType::Dependency => Color::YELLOW,
                ConnectionType::Similarity => Color::PURPLE,
            };
            
            // Line thickness based on bandwidth
            let thickness = 0.1 + conn.data_flow.bandwidth * 0.5;
            
            // Draw connection line
            gizmos.line(from_pos, to_pos, color);
            
            // Animated data packets
            for packet in &conn.data_flow.packets {
                let packet_pos = from_pos.lerp(to_pos, packet.position);
                gizmos.sphere(packet_pos, Quat::IDENTITY, 0.1, Color::WHITE);
            }
        }
    }
}
```

---

## 🎯 **Complete System Architecture**

### Example Scene Setup

```rust
fn setup_visualization_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // 1. Spawn Processing Blocks (Boxes)
    let inference_block = spawn_processing_block(
        &mut commands,
        &mut meshes,
        &mut materials,
        &ProcessingBlock {
            id: "inference_1".to_string(),
            label: "Geometric\nInference".to_string(),
            block_type: BlockType::Inference,
            state: ProcessingState::Idle,
            position: Vec3::new(0.0, 0.0, 0.0),
            inputs: vec![],
            outputs: vec![],
        },
    );
    
    // 2. Spawn Database Nodes (Cylinders)
    let database = spawn_database_node(
        &mut commands,
        &mut meshes,
        &mut materials,
        &DatabaseNode {
            id: "spatial_db_1".to_string(),
            connection: DatabaseConnection {
                connection_string: "postgresql://localhost/spatial".to_string(),
                db_type: DatabaseType::PostgreSQL,
                max_connections: 100,
                timeout: Duration::from_secs(5),
            },
            position: Vec3::new(-5.0, 0.0, 0.0),
            references: vec![],
            status: ConnectionStatus::Connected,
            cache: QueryCache::new(),
        },
    );
    
    // 3. Spawn Node References (Spheres)
    for i in 0..10 {
        let angle = (i as f32 / 10.0) * 2.0 * PI;
        let radius = 3.0;
        
        spawn_node_reference(
            &mut commands,
            &mut meshes,
            &mut materials,
            &NodeReference {
                id: format!("node_{}", i),
                metadata: NodeMetadata {
                    name: format!("Position {}", i),
                    position: i as u8,
                    elp: ELPTensor::new(
                        (angle.cos() + 1.0) / 2.0,
                        (angle.sin() + 1.0) / 2.0,
                        0.5,
                    ),
                    created_at: Utc::now(),
                    last_accessed: Utc::now(),
                    access_count: 0,
                    properties: HashMap::new(),
                    tags: vec!["flux".to_string()],
                },
                ref_type: ReferenceType::FluxNode,
                position: Vec3::new(
                    radius * angle.cos(),
                    0.0,
                    radius * angle.sin(),
                ),
                connections: vec![],
                visible: true,
            },
        );
    }
    
    // 4. Create connections
    commands.spawn(Connection {
        from: database,
        to: inference_block,
        connection_type: ConnectionType::DataFlow,
        data_flow: DataFlow {
            bandwidth: 0.8,
            latency: Duration::from_millis(10),
            packets: vec![],
        },
        active: true,
    });
}
```

---

## 📊 **Shape Legend**

| Shape | Purpose | Color Coding | Size Meaning | Interaction |
|-------|---------|--------------|--------------|-------------|
| **📦 Box** | Processing blocks | State-based | Fixed | Click to inspect |
| **🗄️ Cylinder** | Database | Connection status | Data volume | Query interface |
| **⚪ Sphere** | Node references | ELP tensor (RGB) | Access frequency | Metadata view |
| **➖ Line** | Connections | Connection type | Bandwidth | Trace data flow |

---

## 🎮 **User Interactions**

### Click on Box (Processing Block)
- Show processing logs
- Display input/output data
- Control execution (start/stop/pause)
- View performance metrics

### Click on Cylinder (Database)
- Show connection status
- Display cached queries
- Manual query interface
- View data references

### Click on Sphere (Node Reference)
- Display full metadata
- Show connections
- Edit properties
- View access history

### Click on Line (Connection)
- Show data packets
- Display throughput metrics
- Toggle connection
- Trace data lineage

---

## ✅ Implementation Checklist

- [x] Define shape component structures
- [x] Implement spawn functions
- [x] Create visual properties system
- [x] Add connection visualization
- [x] Document interaction patterns
- [ ] Integrate with geometric_inference
- [ ] Add real-time database queries
- [ ] Implement metadata updates
- [ ] Create UI overlays
- [ ] Add animation systems

---

**This architecture provides a clear visual language for understanding the SpatialVortex system in 3D space** 🎨✨
