# Client Architecture Overview

## ECS Model (Entity-Component-System)

Bevy uses an Entity-Component-System architecture:

- **Entities**: Unique IDs (like database primary keys)
- **Components**: Data attached to entities (position, health, etc.)
- **Systems**: Functions that process entities with specific components

```rust
// Component - just data
#[derive(Component)]
struct Health(f32);

// System - processes entities with Health
fn damage_system(mut query: Query<&mut Health>) {
    for mut health in query.iter_mut() {
        health.0 -= 1.0;
    }
}
```

## Data Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                         BEVY CLIENT                             │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   INPUT                                                         │
│   ─────                                                         │
│   Keyboard (WASD) ──> handle_player_input ──> Reducer Call      │
│                             │                                   │
│                             │ (20Hz throttled)                  │
│                             v                                   │
│   ┌───────────────────────────────────────────────────────┐    │
│   │              SPACETIMEDB CONNECTION                    │    │
│   │  - Sends reducer calls to server                       │    │
│   │  - Receives table updates via WebSocket               │    │
│   └───────────────────────────────────────────────────────┘    │
│                             │                                   │
│                             v                                   │
│   SYNCHRONIZATION                                               │
│   ───────────────                                               │
│   sync_entities_from_db                                         │
│       │                                                         │
│       ├──> Spawn new entities (if in DB but not in world)      │
│       ├──> Update PositionInterpolation targets                 │
│       └──> Despawn removed entities                             │
│                             │                                   │
│                             v                                   │
│   RENDERING                                                     │
│   ─────────                                                     │
│   interpolate_positions (60fps)                                 │
│       │                                                         │
│       └──> Smoothly move Transform toward target                │
│                             │                                   │
│                             v                                   │
│   follow_local_player ──> Camera follows player                 │
│                             │                                   │
│                             v                                   │
│   Bevy Rendering Pipeline ──> Screen                            │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## Network Synchronization Model

### Server-Authoritative Design
- Server owns the "true" position of all entities
- Client sends *intent* (direction), not position
- Client interpolates between server positions for smooth visuals

### Interpolation
```
Server: 20Hz updates (50ms apart)
Client: 60fps rendering

Server positions:   P1 ──────────── P2 ──────────── P3
                     │              │              │
                  50ms           50ms           50ms

Client renders:   P1 → → → → → → P2 → → → → → → P3
                     └─ interpolate smoothly ─┘
```

### The PositionInterpolation Component
```rust
#[derive(Component)]
pub struct PositionInterpolation {
    pub target_position: Vec3,  // Where server says we should be
    pub speed: f32,             // How fast to interpolate
}
```

The `sync_entities_from_db` system updates `target_position`.
The `interpolate_positions` system smoothly moves `Transform` toward it.

## Entity Lifecycle

### Spawn (New Player Joins)
1. Server adds row to `entity_position` table
2. SpacetimeDB pushes update to all clients
3. `sync_entities_from_db` detects new identity
4. System spawns Bevy entity with:
   - `Mesh3d` (capsule shape)
   - `MeshMaterial3d` (blue color)
   - `Transform` (initial position)
   - `NetworkPlayer` (stores Identity)
   - `PositionInterpolation` (for smooth movement)
   - `LocalPlayer` (only if this is our player)

### Update (Player Moves)
1. Server updates `entity_position` row
2. SpacetimeDB pushes update
3. `sync_entities_from_db` finds existing entity
4. Updates `PositionInterpolation.target_position`
5. `interpolate_positions` smoothly moves Transform

### Despawn (Player Leaves)
1. Server removes row from `entity_position`
2. SpacetimeDB pushes deletion
3. `sync_entities_from_db` detects missing identity
4. Calls `commands.entity(e).despawn_recursive()`

## Resource Management

### Connection Resource
```rust
#[derive(Resource)]
pub struct Connection(pub DbConnection);
```
Holds the SpacetimeDB connection. Access via `Res<Connection>`.

### Input Throttle Resource
```rust
#[derive(Resource)]
pub struct InputThrottle {
    timer: Timer,
}
```
Prevents sending input faster than 20Hz (matches server tick rate).

## System Scheduling

```rust
App::new()
    .add_systems(Startup, (
        setup_camera,           // Create camera
        setup_scene,            // Create ground, lights
        setup_connection,       // Subscribe to tables
    ))
    .add_systems(Update, (
        poll_connection,        // Process network messages
        sync_entities_from_db,  // Sync DB → Bevy entities
        handle_player_input,    // Send input to server
        interpolate_positions,  // Smooth movement
        follow_local_player,    // Camera follow
    ))
```

All Update systems run every frame. The order within Update is not guaranteed unless explicitly specified.
