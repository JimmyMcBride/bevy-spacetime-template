# Server Architecture Overview

## Client-Server Model

This game uses a **server-authoritative** architecture where:

1. **Server owns all game state** - Positions, player data, and game logic live on the server
2. **Clients send intent** - Clients tell the server what they want to do (direction, actions)
3. **Server validates and updates** - Server processes inputs, runs physics, updates state
4. **Clients receive state** - Clients render what the server tells them

### Why Server-Authoritative?

- **Prevents cheating** - Clients can't modify their position or other game state
- **Consistent game state** - All players see the same world
- **Easier synchronization** - Single source of truth

## WebSocket Communication

SpacetimeDB handles all networking automatically:

```
Client                          Server
  |                               |
  |-- Reducer Call (JSON) ------->|
  |                               |-- Process Reducer
  |                               |-- Update Tables
  |<-- Table Updates (Binary) ----|
  |                               |
```

- **Reducers**: RPC-like functions clients can call
- **Tables**: Database tables that sync to subscribed clients
- **Subscriptions**: Clients subscribe to queries and receive updates

## Table Concepts

Tables in SpacetimeDB are like database tables that automatically sync:

```rust
#[table(name = entity_position, public)]
pub struct EntityPosition {
    #[primary_key]
    pub owner: Identity,     // Unique identifier
    pub x: f32,
    pub y: f32,
    pub z: f32,
    // ...
}
```

- `public` - All clients can subscribe to this table
- `#[primary_key]` - Unique identifier for each row
- Changes are automatically pushed to subscribed clients

## Reducer Concepts

Reducers are functions that modify the database:

```rust
#[reducer]
pub fn my_reducer(ctx: &ReducerContext, arg: String) -> Result<(), String> {
    // ctx.sender - Identity of the caller
    // ctx.db - Access to all tables
    Ok(())
}
```

- Called by clients via the SDK
- Run atomically (all or nothing)
- Can return errors to reject invalid operations

## Scheduled Tables

For game loops, use scheduled tables:

```rust
#[table(name = movement_timer, scheduled(tick_movement))]
pub struct MovementTimer {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub scheduled_at: ScheduleAt,
}

#[reducer]
pub fn tick_movement(ctx: &ReducerContext, _timer: MovementTimer) -> Result<(), String> {
    // Game loop logic here
    // Re-schedule for next tick
    ctx.db.movement_timer().try_insert(MovementTimer {
        id: 0,
        scheduled_at: Duration::from_millis(50).into(),
    }).ok();
    Ok(())
}
```

This creates a 20Hz game loop (50ms intervals).

## Data Flow Diagram

```
                    SpacetimeDB Server
                    ==================

Client A Input      +----------------+     Client B Input
    |               |                |          |
    v               |   Database     |          v
update_player_input |   --------     | update_player_input
    |               |   player       |          |
    +-------------->|   entity_pos   |<---------+
                    |                |
                    +-------+--------+
                            |
                    tick_movement (20Hz)
                            |
                            v
                    +----------------+
                    | Position       |
                    | Updates        |
                    +-------+--------+
                           /|\
                          / | \
                         /  |  \
                        v   v   v
                    Client A  Client B  Client C
                    (render)  (render)  (render)
```
