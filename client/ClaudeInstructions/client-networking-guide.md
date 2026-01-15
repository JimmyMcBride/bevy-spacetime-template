# Client Networking Guide

## SpacetimeDB SDK Setup

### Connection Resource
```rust
#[derive(Resource)]
pub struct Connection(pub DbConnection);
```

### Creating Connection
```rust
fn create_connection() -> Connection {
    let connection = module_bindings::DbConnection::builder()
        .with_uri("https://maincloud.spacetimedb.com")
        .with_module_name("YOUR_DATABASE_NAME")
        .build()
        .expect("Failed to connect to SpacetimeDB");
    Connection(connection)
}
```

### Inserting as Resource
```rust
fn main() {
    App::new()
        .insert_resource(create_connection())
        // ...
}
```

## Subscription Patterns

### Subscribe to Tables
```rust
pub fn setup_connection(connection: Res<Connection>) {
    let conn = &connection.0;

    // Subscribe to player table
    conn.subscription_builder()
        .subscribe(["SELECT * FROM player"]);

    // Subscribe to entity_position table
    conn.subscription_builder()
        .subscribe(["SELECT * FROM entity_position"]);
}
```

### Filtered Subscriptions
```rust
// Only online players
conn.subscription_builder()
    .subscribe(["SELECT * FROM player WHERE online = true"]);

// Specific player
conn.subscription_builder()
    .subscribe([format!("SELECT * FROM player WHERE identity = {:?}", my_identity)]);
```

## Polling and frame_tick()

### Every Frame Polling
```rust
pub fn poll_connection(connection: Res<Connection>) {
    connection.0.frame_tick();
}
```

This MUST be called every frame to:
- Process incoming WebSocket messages
- Update local table cache
- Trigger callbacks

Without this, the client won't receive updates!

## Entity Sync Implementation

### Full Sync Pattern
```rust
pub fn sync_entities_from_db(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    connection: Res<Connection>,
    mut query: Query<(Entity, &NetworkPlayer, &mut PositionInterpolation)>,
) {
    let local_identity = connection.0.try_identity();
    let db_positions: Vec<EntityPosition> = connection.0.db.entity_position().iter().collect();

    // Track seen identities to detect removals
    let mut seen_identities = std::collections::HashSet::new();

    for pos in &db_positions {
        seen_identities.insert(pos.owner);

        // Find or spawn
        let existing = query.iter_mut().find(|(_, np, _)| np.identity == pos.owner);

        if let Some((_, _, mut interp)) = existing {
            // UPDATE existing
            interp.target_position = Vec3::new(pos.x, 0.5, pos.z);
        } else {
            // SPAWN new
            spawn_player(&mut commands, &mut meshes, &mut materials, pos, local_identity);
        }
    }

    // DESPAWN removed
    for (entity, np, _) in query.iter() {
        if !seen_identities.contains(&np.identity) {
            commands.entity(entity).despawn_recursive();
        }
    }
}
```

### Spawn Helper
```rust
fn spawn_player(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    pos: &EntityPosition,
    local_identity: Option<Identity>,
) {
    let mut entity_commands = commands.spawn((
        Mesh3d(meshes.add(Capsule3d::new(0.3, 1.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.6, 0.8),
            ..default()
        })),
        Transform::from_xyz(pos.x, 0.5, pos.z),
        NetworkPlayer { identity: pos.owner },
        PositionInterpolation {
            target_position: Vec3::new(pos.x, 0.5, pos.z),
            speed: 10.0,
        },
    ));

    // Mark local player
    if Some(pos.owner) == local_identity {
        entity_commands.insert(LocalPlayer);
    }
}
```

## Reducer Invocation

### Basic Call
```rust
fn my_system(connection: Res<Connection>) {
    let _ = connection.0.reducers.create_player("PlayerName".to_string());
}
```

### With Arguments
```rust
let _ = connection.0.reducers.update_player_input(
    direction.x,   // f32
    direction.z,   // f32
    moving,        // bool
);
```

### Handling Results
```rust
match connection.0.reducers.create_player("Name".to_string()) {
    Ok(_) => println!("Player created"),
    Err(e) => println!("Failed: {:?}", e),
}
```

Note: Reducer calls are asynchronous. The Result indicates if the call was sent, not if it succeeded on the server.

## Getting Local Identity

```rust
// Option<Identity> - None if not connected yet
let local_identity = connection.0.try_identity();

// Check if this entity is ours
if Some(entity.owner) == local_identity {
    // This is our player
}
```

## Binding Regeneration

After server schema changes:

```bash
cd server
spacetime generate --lang rust --out-dir ../client/src/module_bindings --project-path .
```

This generates:
- `module_bindings/mod.rs` - Main module
- Table type definitions
- Reducer function bindings
- `DbConnection` builder

## Table Access

### Iterate All Rows
```rust
for entity in connection.0.db.entity_position().iter() {
    println!("Position: {}, {}", entity.x, entity.z);
}
```

### Collect to Vec
```rust
let positions: Vec<EntityPosition> = connection.0.db.entity_position().iter().collect();
```

### Find by Primary Key
```rust
use crate::module_bindings::entity_position_table::EntityPositionTableAccess;

if let Some(pos) = connection.0.db.entity_position().owner().find(&identity) {
    // Found
}
```

## Troubleshooting

### "Connection failed"
- Check database name matches published name
- Verify server URI (maincloud vs testnet vs local)
- Check internet connection

### "Table is empty"
- Ensure `frame_tick()` is being called
- Check subscription was set up
- Verify server has data (use `spacetime sql`)

### "Entities not updating"
- Verify `poll_connection` system is registered
- Check `sync_entities_from_db` is running
- Ensure subscription query matches data

### "Local player not marked"
- `try_identity()` may return None initially
- Add a fallback to mark local player later:
```rust
// If we have a NetworkPlayer without LocalPlayer, and it matches our identity
fn mark_local_player(
    mut commands: Commands,
    connection: Res<Connection>,
    query: Query<(Entity, &NetworkPlayer), Without<LocalPlayer>>,
) {
    let Some(local_id) = connection.0.try_identity() else { return };

    for (entity, np) in query.iter() {
        if np.identity == local_id {
            commands.entity(entity).insert(LocalPlayer);
        }
    }
}
```

### "Bindings don't match server"
Regenerate bindings after any server change:
```bash
cd server
spacetime generate --lang rust --out-dir ../client/src/module_bindings --project-path .
```
