# Bevy + SpacetimeDB Multiplayer Game Template

A production-ready template for creating server-authoritative multiplayer games with Bevy 0.15 and SpacetimeDB 1.0.

## Features

- **Server-authoritative architecture** - All game state lives on the server, preventing cheating
- **20Hz server tick rate** - Server runs game logic at 50ms intervals
- **60fps client rendering** - Smooth interpolation between server updates
- **Automatic synchronization** - SpacetimeDB handles real-time state sync via WebSockets
- **Modular documentation** - Token-efficient CLAUDE.md structure for AI-assisted development

---

## Quick Start

### Prerequisites

```bash
# 1. Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 2. WASM target for SpacetimeDB modules
rustup target add wasm32-unknown-unknown

# 3. SpacetimeDB CLI
curl -sSf https://install.spacetimedb.com | sh
```

### Step 1: Build and Publish the Server

```bash
cd server
cargo build --target=wasm32-unknown-unknown --release
spacetime publish YOUR_DB_NAME --server maincloud
```

Replace `YOUR_DB_NAME` with your chosen database name (e.g., `my-awesome-game`).

### Step 2: Generate Client Bindings

```bash
cd server
spacetime generate --lang rust --out-dir ../client/src/module_bindings --project-path .
```

This creates the `module_bindings/` directory with auto-generated Rust types matching your server schema.

### Step 3: Configure the Client

Edit `client/src/main.rs` line 30 - change `YOUR_DATABASE_NAME` to match the name you used in Step 1:

```rust
.with_module_name("my-awesome-game")  // <-- Your database name here
```

### Step 4: Run the Client

```bash
cd client
cargo run
```

Use WASD or arrow keys to move your player around.

---

## How This Template Works

### Architecture Overview

```
┌─────────────────┐         ┌─────────────────────────────────┐
│   BEVY CLIENT   │         │       SPACETIMEDB SERVER        │
│                 │         │                                 │
│  ┌───────────┐  │  WASD   │  ┌─────────────────────────┐   │
│  │   Input   │──┼────────►│  │  update_player_input()  │   │
│  │  System   │  │         │  │  - Store direction      │   │
│  └───────────┘  │         │  └─────────────────────────┘   │
│                 │         │              │                  │
│                 │         │              ▼                  │
│                 │         │  ┌─────────────────────────┐   │
│                 │         │  │   tick_movement (20Hz)  │   │
│                 │         │  │   - Apply velocity      │   │
│                 │         │  │   - Update positions    │   │
│                 │         │  └─────────────────────────┘   │
│                 │         │              │                  │
│                 │  WebSocket Push        │                  │
│  ┌───────────┐  │◄─────────────────────┘                  │
│  │   Sync    │  │         │                                 │
│  │  System   │  │         │  Tables:                        │
│  └─────┬─────┘  │         │  - player (identity, name)      │
│        │        │         │  - entity_position (x, y, z)    │
│        ▼        │         │  - movement_timer (scheduler)   │
│  ┌───────────┐  │         │                                 │
│  │Interpolate│  │         └─────────────────────────────────┘
│  │  (60fps)  │  │
│  └─────┬─────┘  │
│        │        │
│        ▼        │
│  ┌───────────┐  │
│  │  Render   │  │
│  └───────────┘  │
└─────────────────┘
```

### Key Principle: Clients Send Intent, Server Calculates State

**Traditional (client-authoritative) approach:**
```
Client: "My position is now (10, 0, 5)"  ← Easy to cheat!
```

**This template (server-authoritative) approach:**
```
Client: "I want to move in direction (1, 0, 0)"
Server: "OK, your new position is (10.25, 0, 5)"  ← Server controls truth
```

### The Server (`server/src/lib.rs`)

The server is a SpacetimeDB module compiled to WebAssembly. It contains:

#### Tables (Database Schema)

```rust
// Player metadata
#[table(name = player, public)]
pub struct Player {
    #[primary_key]
    pub identity: Identity,  // Unique player ID from SpacetimeDB
    pub name: String,
    pub online: bool,
}

// Real-time position data (the core game state)
#[table(name = entity_position, public)]
pub struct EntityPosition {
    #[primary_key]
    pub owner: Identity,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub direction_x: f32,   // Current movement direction
    pub direction_z: f32,
    pub moving: bool,       // Is the player moving?
}

// Scheduler for the game loop
#[table(name = movement_timer, scheduled(tick_movement))]
pub struct MovementTimer { ... }
```

#### Reducers (Server Functions)

```rust
// Called by client when they want to move
#[reducer]
pub fn update_player_input(ctx, direction_x, direction_z, moving) {
    // Just stores the INTENT, doesn't move the player
    position.direction_x = direction_x;
    position.direction_z = direction_z;
    position.moving = moving;
}

// Game loop - runs every 50ms (20Hz)
#[reducer]
pub fn tick_movement(ctx, _timer) {
    for entity in all_entities {
        if entity.moving {
            // Server calculates the actual movement
            entity.x += entity.direction_x * SPEED * DELTA_TIME;
            entity.z += entity.direction_z * SPEED * DELTA_TIME;
        }
    }
    // Re-schedule for next tick
}
```

### The Client (`client/src/`)

The client is a Bevy 0.15 application with these key systems:

#### Network Systems (`systems/network.rs`)

```rust
// Called every frame - processes incoming WebSocket messages
pub fn poll_connection(connection: Res<Connection>) {
    connection.0.frame_tick();  // CRITICAL: Must be called!
}

// Syncs database state to Bevy entities
pub fn sync_entities_from_db(...) {
    // For each row in entity_position table:
    //   - If no Bevy entity exists → Spawn one
    //   - If entity exists → Update interpolation target
    //   - If row was deleted → Despawn entity
}
```

#### Input System (`systems/input.rs`)

```rust
// Throttled to 20Hz to match server tick rate
pub fn handle_player_input(...) {
    let direction = /* calculate from WASD */;

    // Send intent to server (not position!)
    connection.reducers.update_player_input(
        direction.x,
        direction.z,
        moving,
    );
}
```

#### Movement System (`systems/movement.rs`)

```rust
// Runs at 60fps for smooth visuals
pub fn interpolate_positions(...) {
    for (transform, interpolation) in query {
        // Smoothly move toward the server-provided target
        transform.translation = lerp(
            transform.translation,
            interpolation.target_position,
            smoothing_factor,
        );
    }
}
```

### Why Interpolation?

Server updates come at 20Hz (every 50ms), but we render at 60fps (every 16ms). Without interpolation, movement would be jerky:

```
Without interpolation:
Frame 1-3:  Position = (0, 0, 0)     ← Stuck for 50ms
Frame 4:    Position = (0.25, 0, 0)  ← Sudden jump!
Frame 5-7:  Position = (0.25, 0, 0)  ← Stuck again

With interpolation:
Frame 1:    Position = (0, 0, 0)
Frame 2:    Position = (0.08, 0, 0)   ← Smooth movement
Frame 3:    Position = (0.16, 0, 0)   ← toward target
Frame 4:    Position = (0.25, 0, 0)   ← Arrives smoothly
```

---

## Customizing the Template

### Changing Game Constants

Edit `server/src/lib.rs`:

```rust
// In tick_movement reducer
const SPEED: f32 = 5.0;        // Units per second (increase for faster movement)
const DELTA_TIME: f32 = 0.05;  // Don't change unless changing tick rate
const WORLD_BOUNDS: f32 = 50.0; // Play area size
```

### Adding New Player Properties

**Step 1: Add to server table**

```rust
#[table(name = entity_position, public)]
pub struct EntityPosition {
    // ... existing fields ...
    pub health: f32,        // NEW
    pub score: u32,         // NEW
}
```

**Step 2: Initialize in create_player reducer**

```rust
ctx.db.entity_position().try_insert(EntityPosition {
    // ... existing fields ...
    health: 100.0,
    score: 0,
})?;
```

**Step 3: Regenerate bindings**

```bash
cd server
cargo build --target=wasm32-unknown-unknown --release
spacetime publish YOUR_DB_NAME --server maincloud
spacetime generate --lang rust --out-dir ../client/src/module_bindings --project-path .
```

**Step 4: Use in client**

```rust
// In sync_entities_from_db or a new system
for pos in &db_positions {
    println!("Player health: {}", pos.health);
}
```

### Adding New Actions (e.g., Shooting)

**Step 1: Add server reducer**

```rust
#[reducer]
pub fn player_shoot(ctx: &ReducerContext, target_x: f32, target_z: f32) -> Result<(), String> {
    let shooter = ctx.db.entity_position().owner().find(&ctx.sender)
        .ok_or("Player not found")?;

    // Create projectile, check hits, etc.
    // All game logic happens HERE on the server

    Ok(())
}
```

**Step 2: Regenerate bindings** (same as above)

**Step 3: Call from client**

```rust
// In a new input handler
if mouse.just_pressed(MouseButton::Left) {
    let _ = connection.0.reducers.player_shoot(target_x, target_z);
}
```

### Adding New Entity Types

**Step 1: Create server table**

```rust
#[table(name = projectile, public)]
pub struct Projectile {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub x: f32,
    pub z: f32,
    pub velocity_x: f32,
    pub velocity_z: f32,
    pub owner: Identity,
}
```

**Step 2: Update tick_movement to process projectiles**

```rust
#[reducer]
pub fn tick_movement(ctx: &ReducerContext, _timer: MovementTimer) -> Result<(), String> {
    // Existing player movement...

    // NEW: Process projectiles
    for mut proj in ctx.db.projectile().iter() {
        proj.x += proj.velocity_x * DELTA_TIME;
        proj.z += proj.velocity_z * DELTA_TIME;
        ctx.db.projectile().id().update(proj);

        // Check for collisions, despawn if out of bounds, etc.
    }

    Ok(())
}
```

**Step 3: Create client component and sync system**

```rust
// components.rs
#[derive(Component)]
pub struct ProjectileMarker {
    pub id: u64,
}

// In network.rs or new file
pub fn sync_projectiles(...) {
    // Similar pattern to sync_entities_from_db
}
```

### Changing Player Appearance

Edit `client/src/systems/network.rs` in `sync_entities_from_db`:

```rust
// Change mesh shape
Mesh3d(meshes.add(Cuboid::new(0.5, 1.0, 0.5))),  // Box instead of capsule

// Change color
MeshMaterial3d(materials.add(StandardMaterial {
    base_color: Color::srgb(1.0, 0.5, 0.0),  // Orange
    metallic: 0.8,
    ..default()
})),
```

### Adding a Different Camera Style

Edit `client/src/systems/camera.rs`:

```rust
// For a top-down camera
const CAMERA_OFFSET: Vec3 = Vec3::new(0.0, 20.0, 0.0);

pub fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 20.0, 0.0).looking_at(Vec3::ZERO, Vec3::NEG_Z),
    ));
}
```

---

## Project Structure

```
bevy-spacetime-template/
├── README.md                        # This file
├── CLAUDE.md                        # AI assistant documentation hub
├── TEMPLATE_INSTRUCTIONS.md         # Original template spec
│
├── client/                          # Bevy game client
│   ├── Cargo.toml
│   ├── CLAUDE.md                    # Client-specific docs
│   ├── ClaudeInstructions/          # Detailed guides
│   │   ├── architecture-overview.md
│   │   ├── bevy-systems-guide.md
│   │   ├── client-networking-guide.md
│   │   ├── coding-standards.md
│   │   └── common-tasks.md
│   ├── assets/
│   │   └── textures/
│   └── src/
│       ├── main.rs                  # App entry point
│       ├── components.rs            # ECS components
│       ├── module_bindings/         # Auto-generated (do not edit!)
│       └── systems/
│           ├── mod.rs
│           ├── camera.rs            # Camera setup & follow
│           ├── input.rs             # WASD handling
│           ├── movement.rs          # Position interpolation
│           └── network.rs           # SpacetimeDB sync
│
└── server/                          # SpacetimeDB module
    ├── Cargo.toml
    ├── CLAUDE.md                    # Server-specific docs
    ├── ClaudeInstructions/          # Detailed guides
    │   ├── architecture-overview.md
    │   ├── build-deployment-guide.md
    │   ├── coding-standards.md
    │   ├── common-tasks.md
    │   └── spacetimedb-module-guide.md
    └── src/
        └── lib.rs                   # Tables & reducers
```

---

## Common Commands Reference

```bash
# === SERVER ===
cd server
cargo build --target=wasm32-unknown-unknown --release  # Build
spacetime publish DB_NAME --server maincloud           # Deploy
spacetime logs DB_NAME --server maincloud -f           # View logs

# === BINDINGS ===
cd server
spacetime generate --lang rust --out-dir ../client/src/module_bindings --project-path .

# === CLIENT ===
cd client
cargo run                                              # Debug build
cargo build --release                                  # Release build
cargo clippy                                           # Lint

# === CROSS-COMPILE FOR WINDOWS ===
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu
```

---

## Troubleshooting

### "Connection failed" / "Module not found"
- Verify database name in `client/src/main.rs` matches what you published
- Check that the server was published successfully: `spacetime list --server maincloud`

### "Entities not appearing"
- Ensure `poll_connection` system is registered and running
- Check that subscriptions were set up in `setup_connection`
- Verify server has data: `spacetime sql DB_NAME --server maincloud "SELECT * FROM entity_position"`

### "Movement is jerky"
- Make sure `interpolate_positions` system is running
- Check that `PositionInterpolation` component is being added to entities

### "Changes to server not reflected"
- Rebuild and republish the server
- Regenerate client bindings
- Restart the client

---

## License

MIT License - Use this template for any project, commercial or otherwise.
