# Client Coding Standards

## Naming Conventions

### Rust Standard
- **Structs/Components**: `PascalCase` (e.g., `LocalPlayer`, `PositionInterpolation`)
- **Functions/Systems**: `snake_case` (e.g., `handle_player_input`, `sync_entities_from_db`)
- **Constants**: `SCREAMING_SNAKE_CASE` (e.g., `CAMERA_OFFSET`, `SPEED`)
- **Modules**: `snake_case` (e.g., `network`, `movement`)

### Bevy Conventions
- **Components**: Noun or adjective (e.g., `LocalPlayer`, `Dead`, `Visible`)
- **Systems**: Verb phrase (e.g., `handle_player_input`, `follow_local_player`)
- **Resources**: Noun (e.g., `Connection`, `InputThrottle`)

## File Organization

```
client/
├── src/
│   ├── main.rs              # App setup, plugins, scene
│   ├── components.rs        # All ECS components
│   ├── systems/
│   │   ├── mod.rs           # Module exports
│   │   ├── network.rs       # SpacetimeDB sync
│   │   ├── input.rs         # Player input
│   │   ├── movement.rs      # Position interpolation
│   │   └── camera.rs        # Camera follow
│   └── module_bindings/     # Auto-generated (do not edit)
├── assets/
│   └── textures/
├── Cargo.toml
└── CLAUDE.md
```

### Module Exports Pattern
```rust
// systems/mod.rs
pub mod camera;
pub mod input;
pub mod movement;
pub mod network;
```

### Main.rs Imports
```rust
mod components;
mod systems;
mod module_bindings;

use bevy::prelude::*;
use systems::*;
```

## Bevy-Specific Patterns

### Component Definition
```rust
use bevy::prelude::*;

/// Brief description of what this marks/stores
#[derive(Component)]
pub struct MyComponent {
    pub field: f32,
}

// Marker components (no data)
#[derive(Component)]
pub struct LocalPlayer;
```

### System Definition
```rust
/// Brief description of what this system does
pub fn my_system(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<LocalPlayer>>,
) {
    for mut transform in query.iter_mut() {
        // Implementation
    }
}
```

### Resource Definition
```rust
#[derive(Resource)]
pub struct MyResource {
    pub data: String,
}

impl Default for MyResource {
    fn default() -> Self {
        Self {
            data: "default".to_string(),
        }
    }
}
```

## System Ordering

### Startup Systems
Group related setup:
```rust
.add_systems(Startup, (
    camera::setup_camera,
    setup_scene,
    network::setup_connection,
))
```

### Update Systems
Order by data flow:
```rust
.add_systems(Update, (
    // 1. Network (get data)
    network::poll_connection,
    network::sync_entities_from_db,
    // 2. Input (process player intent)
    input::handle_player_input,
    // 3. Movement (update positions)
    movement::interpolate_positions,
    // 4. Camera (follow player)
    camera::follow_local_player,
))
```

## Query Best Practices

### Use Specific Queries
```rust
// GOOD - Only what you need
Query<&Transform, With<LocalPlayer>>

// AVOID - Over-fetching
Query<(&Transform, &NetworkPlayer, &PositionInterpolation, &PlayerCapsule)>
```

### Handle Single Entity Queries
```rust
// Prefer get_single() with early return
let Ok(transform) = query.get_single() else { return };

// Instead of single() which panics
let transform = query.single(); // Don't use unless you're sure
```

### Mutable vs Immutable
```rust
// Only use mut when modifying
Query<&Transform>       // Read-only
Query<&mut Transform>   // When changing values
```

## Error Handling

### System Early Returns
```rust
pub fn follow_local_player(
    player_query: Query<&Transform, With<LocalPlayer>>,
    mut camera_query: Query<&mut Transform, With<Camera3d>>,
) {
    // Early return if queries fail
    let Ok(player_transform) = player_query.get_single() else { return };
    let Ok(mut camera_transform) = camera_query.get_single_mut() else { return };

    // Rest of implementation
}
```

### Network Calls
```rust
// Ignore errors for fire-and-forget calls
let _ = connection.0.reducers.update_player_input(x, z, moving);

// Or handle if needed
if let Err(e) = connection.0.reducers.create_player(name) {
    eprintln!("Failed to create player: {:?}", e);
}
```

## Constants

### Define at Module Level
```rust
// camera.rs
const CAMERA_OFFSET: Vec3 = Vec3::new(10.0, 10.0, 10.0);

// input.rs
const INPUT_RATE: f32 = 0.05; // 20Hz

// movement.rs
const INTERPOLATION_SPEED: f32 = 10.0;
```

## Documentation Standards

### Component Documentation
```rust
/// Marks the local player controlled by this client.
///
/// Added to the entity representing this client's player.
/// Used to identify which entity should receive input and camera follow.
#[derive(Component)]
pub struct LocalPlayer;
```

### System Documentation
```rust
/// Smooth interpolation for 60fps visuals from 20Hz server updates.
///
/// Uses exponential smoothing that is framerate-independent.
/// The smoothing factor is calculated as `1 - e^(-speed * dt)`.
pub fn interpolate_positions(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &PositionInterpolation)>,
) {
    // ...
}
```

## Anti-Patterns to Avoid

### Don't Panic in Systems
```rust
// BAD - Panics if no local player
let transform = player_query.single();

// GOOD - Graceful handling
let Ok(transform) = player_query.get_single() else { return };
```

### Don't Duplicate State
```rust
// BAD - Storing position in component AND transform
#[derive(Component)]
pub struct Player {
    pub position: Vec3,  // Duplicates Transform
}

// GOOD - Use Transform, interpolation is separate concern
#[derive(Component)]
pub struct PositionInterpolation {
    pub target_position: Vec3,  // Where we're heading
}
```

### Don't Over-Engineer
```rust
// BAD - Over-abstracted
trait EntitySynchronizer { ... }
impl EntitySynchronizer for PlayerSync { ... }

// GOOD - Direct and simple
pub fn sync_entities_from_db( ... ) {
    // Just do the work directly
}
```

### Don't Forget frame_tick()
```rust
// BAD - Network never updates
// (missing poll_connection system)

// GOOD - Always poll
.add_systems(Update, network::poll_connection)
```
