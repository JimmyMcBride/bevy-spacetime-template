# Client Documentation

> **Token Conservation**: Only read guides from `ClaudeInstructions/` relevant to your current task.

## Overview

Bevy 0.15 game client with SpacetimeDB networking.

## Quick Commands

```bash
cargo run                    # Debug build
cargo build --release        # Release build
cargo clippy                 # Lint
```

## ECS Components

| Component | Purpose |
|-----------|---------|
| `LocalPlayer` | Marks player controlled by this client |
| `NetworkPlayer` | Stores SpacetimeDB identity |
| `PositionInterpolation` | Smooth movement data |
| `PlayerCapsule` | Visual mesh marker |

## Systems

| System | Schedule | Purpose |
|--------|----------|---------|
| `setup_camera` | Startup | Isometric camera |
| `setup_connection` | Startup | SpacetimeDB subscriptions |
| `poll_connection` | Update | Process network updates |
| `sync_entities_from_db` | Update | Spawn/update/despawn entities |
| `handle_player_input` | Update | WASD input (20Hz throttled) |
| `interpolate_positions` | Update | Smooth movement (60fps) |
| `follow_local_player` | Update | Camera follow |

## Common Patterns

### Query entities
```rust
fn my_system(query: Query<&Transform, With<LocalPlayer>>) {
    for transform in query.iter() { /* ... */ }
}
```

### Call reducer
```rust
fn my_system(connection: Res<Connection>) {
    let _ = connection.0.reducers.update_player_input(x, z, moving);
}
```

## Guides (ClaudeInstructions/)

| Guide | Read When |
|-------|-----------|
| architecture-overview.md | Understanding system design |
| bevy-systems-guide.md | Adding/modifying systems |
| client-networking-guide.md | Network sync issues |
| coding-standards.md | Writing new code |
| common-tasks.md | Step-by-step workflows |
