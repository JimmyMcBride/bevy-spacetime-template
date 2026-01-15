# Server Coding Standards

## Rust Style Guidelines

### Naming Conventions
- **Structs**: `PascalCase` (e.g., `EntityPosition`, `MovementTimer`)
- **Functions/Reducers**: `snake_case` (e.g., `create_player`, `update_player_input`)
- **Constants**: `SCREAMING_SNAKE_CASE` (e.g., `SPEED`, `WORLD_BOUNDS`)
- **Fields**: `snake_case` (e.g., `direction_x`, `scheduled_at`)

### Table Naming
```rust
// Table name in attribute should be snake_case
#[table(name = entity_position, public)]
pub struct EntityPosition { ... }
```

### File Organization
```
server/
├── src/
│   └── lib.rs          # All tables and reducers (single file for simplicity)
├── Cargo.toml
└── CLAUDE.md
```

For larger projects, split into modules:
```
server/
├── src/
│   ├── lib.rs          # Module declarations and init
│   ├── tables.rs       # Table definitions
│   ├── reducers/
│   │   ├── mod.rs
│   │   ├── player.rs
│   │   └── movement.rs
│   └── utils.rs
```

## SpacetimeDB Patterns

### Table Sections
Organize tables by purpose with comments:
```rust
// ============================================================================
// TABLES
// ============================================================================

/// Player identity and metadata
#[table(name = player, public)]
pub struct Player { ... }

/// Real-time entity position
#[table(name = entity_position, public)]
pub struct EntityPosition { ... }
```

### Reducer Sections
```rust
// ============================================================================
// REDUCERS
// ============================================================================

/// Create new player (called by client on connect)
#[reducer]
pub fn create_player(ctx: &ReducerContext, name: String) -> Result<(), String> {
    // ...
}
```

### Error Handling Pattern
```rust
#[reducer]
pub fn my_reducer(ctx: &ReducerContext) -> Result<(), String> {
    // Use ? operator with descriptive errors
    let player = ctx.db.player().identity().find(&ctx.sender)
        .ok_or("Player not found")?;

    // Return Err for validation failures
    if player.name.is_empty() {
        return Err("Player name cannot be empty".to_string());
    }

    Ok(())
}
```

### Idempotent Operations
Make reducers safe to call multiple times:
```rust
#[reducer]
pub fn create_player(ctx: &ReducerContext, name: String) -> Result<(), String> {
    // Check if already exists - return Ok, not error
    if ctx.db.player().identity().find(&ctx.sender).is_some() {
        return Ok(()); // Already exists, no-op
    }

    // Create new player
    ctx.db.player().try_insert(Player { ... })?;
    Ok(())
}
```

### Constants
Define game constants at the top of the file:
```rust
const SPEED: f32 = 5.0;        // Units per second
const DELTA_TIME: f32 = 0.05;  // 50ms tick rate
const WORLD_BOUNDS: f32 = 50.0;
const TICK_RATE_MS: u64 = 50;
```

## Documentation Standards

### Struct Documentation
```rust
/// Real-time entity position (server-authoritative)
///
/// Updated by tick_movement reducer at 20Hz.
/// Clients should not modify directly.
#[table(name = entity_position, public)]
pub struct EntityPosition {
    /// Unique owner identity
    #[primary_key]
    pub owner: Identity,
    /// X position in world units
    pub x: f32,
    // ...
}
```

### Reducer Documentation
```rust
/// Update player input direction
///
/// Called by client when WASD keys change.
/// Server normalizes the direction vector.
///
/// # Arguments
/// * `direction_x` - X component of movement direction
/// * `direction_z` - Z component of movement direction
/// * `moving` - Whether player is actively moving
#[reducer]
pub fn update_player_input(
    ctx: &ReducerContext,
    direction_x: f32,
    direction_z: f32,
    moving: bool,
) -> Result<(), String> { ... }
```

## Common Anti-Patterns to Avoid

### Don't Trust Client Data
```rust
// BAD - Client could send any position
#[reducer]
pub fn set_position(ctx: &ReducerContext, x: f32, z: f32) {
    entity.x = x;  // Cheating possible!
    entity.z = z;
}

// GOOD - Server calculates position from input
#[reducer]
pub fn update_player_input(ctx: &ReducerContext, dir_x: f32, dir_z: f32, moving: bool) {
    // Server applies movement in tick_movement
    entity.direction_x = dir_x.clamp(-1.0, 1.0);  // Validate input
    entity.direction_z = dir_z.clamp(-1.0, 1.0);
    entity.moving = moving;
}
```

### Don't Forget to Re-schedule Timers
```rust
// BAD - Timer fires once and stops
#[reducer]
pub fn tick_movement(ctx: &ReducerContext, _timer: MovementTimer) {
    // Do work but forget to re-schedule
}

// GOOD - Timer continues running
#[reducer]
pub fn tick_movement(ctx: &ReducerContext, _timer: MovementTimer) {
    // Do work

    // Re-schedule for next tick
    ctx.db.movement_timer().try_insert(MovementTimer {
        id: 0,
        scheduled_at: Duration::from_millis(50).into(),
    }).ok();
}
```

### Handle Missing Data Gracefully
```rust
// BAD - Panics if player doesn't exist
#[reducer]
pub fn update_player(ctx: &ReducerContext) {
    let player = ctx.db.player().identity().find(&ctx.sender).unwrap();
}

// GOOD - Returns error
#[reducer]
pub fn update_player(ctx: &ReducerContext) -> Result<(), String> {
    let player = ctx.db.player().identity().find(&ctx.sender)
        .ok_or("Player not found")?;
    Ok(())
}
```
