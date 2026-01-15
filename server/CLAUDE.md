# Server Documentation

> **Token Conservation**: Only read guides from `ClaudeInstructions/` relevant to your current task.

## Overview

SpacetimeDB 1.0 module with server-authoritative game logic.

## Quick Commands

```bash
cargo build --target=wasm32-unknown-unknown --release
spacetime publish DB_NAME --server maincloud
spacetime logs DB_NAME --server maincloud
```

## Tables

| Table | Purpose |
|-------|---------|
| `player` | Player identity and metadata |
| `entity_position` | Real-time position (server-authoritative) |
| `movement_timer` | Scheduled tick trigger |

## Reducers

| Reducer | Called By | Purpose |
|---------|-----------|---------|
| `create_player` | Client | New player setup |
| `update_player_input` | Client | Movement input |
| `tick_movement` | Scheduler | Physics at 20Hz |
| `on_client_connected` | System | Online status |
| `on_client_disconnected` | System | Offline status |

## SpacetimeDB 1.0 Patterns

### Define table
```rust
#[table(name = my_table, public)]
pub struct MyTable {
    #[primary_key]
    pub id: u64,
}
```

### Define reducer
```rust
#[reducer]
pub fn my_reducer(ctx: &ReducerContext, arg: String) -> Result<(), String> {
    ctx.db.my_table().try_insert(/* ... */)?;
    Ok(())
}
```

### Database operations
```rust
ctx.db.table().try_insert(row)?;           // Insert
ctx.db.table().field().find(&value);       // Find by PK
ctx.db.table().field().update(row);        // Update
ctx.db.table().iter()                      // Iterate all
```

## Guides (ClaudeInstructions/)

| Guide | Read When |
|-------|-----------|
| architecture-overview.md | Understanding client-server model |
| spacetimedb-module-guide.md | Tables, reducers, scheduled tables |
| build-deployment-guide.md | Publishing and deployment |
| coding-standards.md | Writing new code |
| common-tasks.md | Step-by-step workflows |
