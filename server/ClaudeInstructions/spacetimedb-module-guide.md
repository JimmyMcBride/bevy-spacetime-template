# SpacetimeDB Module Guide

## Table Definition

### Basic Table
```rust
use spacetimedb::{table, Identity};

#[table(name = my_table, public)]
pub struct MyTable {
    #[primary_key]
    pub id: u64,
    pub data: String,
}
```

### Table Attributes
- `name = table_name` - Database table name (required)
- `public` - Clients can subscribe to this table
- `private` - Only server can access (default)

### Field Attributes
- `#[primary_key]` - Unique identifier (required, one per table)
- `#[auto_inc]` - Auto-increment for integer fields
- `#[unique]` - Enforce uniqueness

### Supported Types
- Primitives: `u8`, `u16`, `u32`, `u64`, `i8`, `i16`, `i32`, `i64`, `f32`, `f64`, `bool`
- Strings: `String`
- Identity: `Identity` (SpacetimeDB user identifier)
- Collections: `Vec<T>`, `Option<T>`
- Custom: Other `#[table]` structs (for relations)

## Reducer Definition

### Basic Reducer
```rust
use spacetimedb::{reducer, ReducerContext};

#[reducer]
pub fn my_reducer(ctx: &ReducerContext, arg: String) -> Result<(), String> {
    // Implementation
    Ok(())
}
```

### Special Reducers
```rust
// Called when module is first published
#[reducer(init)]
pub fn init(ctx: &ReducerContext) { }

// Called when client connects
#[reducer(client_connected)]
pub fn on_connect(ctx: &ReducerContext) { }

// Called when client disconnects
#[reducer(client_disconnected)]
pub fn on_disconnect(ctx: &ReducerContext) { }
```

### ReducerContext
```rust
ctx.sender      // Identity of the caller
ctx.timestamp   // When the reducer was called
ctx.db          // Database access
```

## Database Operations

### Insert
```rust
// Try insert (returns Result)
ctx.db.my_table().try_insert(MyTable {
    id: 1,
    data: "hello".to_string(),
})?;

// Insert (panics on error)
ctx.db.my_table().insert(MyTable { ... });
```

### Find by Primary Key
```rust
// Returns Option<MyTable>
let row = ctx.db.my_table().id().find(&1);

if let Some(mut row) = row {
    // Found
}
```

### Update
```rust
// Must have the row first
let mut row = ctx.db.my_table().id().find(&1).ok_or("Not found")?;
row.data = "updated".to_string();
ctx.db.my_table().id().update(row);
```

### Delete
```rust
// Delete by primary key
ctx.db.my_table().id().delete(&1);
```

### Iterate All Rows
```rust
for row in ctx.db.my_table().iter() {
    // Process each row
}
```

### Find by Other Fields
```rust
// If you have a unique field
#[table(name = players, public)]
pub struct Player {
    #[primary_key]
    pub id: u64,
    #[unique]
    pub username: String,
}

// Find by unique field
let player = ctx.db.players().username().find(&"bob".to_string());
```

## Scheduled Tables

### Definition
```rust
use spacetimedb::{table, reducer, ReducerContext, ScheduleAt};
use std::time::Duration;

#[table(name = my_timer, scheduled(on_timer))]
pub struct MyTimer {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub scheduled_at: ScheduleAt,
}

#[reducer]
pub fn on_timer(ctx: &ReducerContext, timer: MyTimer) -> Result<(), String> {
    // Timer fired!

    // Re-schedule for recurring timer
    ctx.db.my_timer().try_insert(MyTimer {
        id: 0,  // auto_inc will assign new ID
        scheduled_at: Duration::from_millis(100).into(),
    }).ok();

    Ok(())
}
```

### Starting the Timer
```rust
#[reducer(init)]
pub fn init(ctx: &ReducerContext) {
    // Start timer on module init
    ctx.db.my_timer().try_insert(MyTimer {
        id: 0,
        scheduled_at: Duration::from_millis(100).into(),
    }).ok();
}
```

### ScheduleAt Options
```rust
// Relative time (from now)
Duration::from_millis(50).into()

// Absolute time
ScheduleAt::Time(timestamp)

// Interval (repeating)
ScheduleAt::Interval(Duration::from_secs(1))
```

## Lifecycle Hooks

### Client Connected
```rust
#[reducer(client_connected)]
pub fn on_client_connected(ctx: &ReducerContext) {
    // ctx.sender is the connecting client's Identity
    println!("Client connected: {:?}", ctx.sender);
}
```

### Client Disconnected
```rust
#[reducer(client_disconnected)]
pub fn on_client_disconnected(ctx: &ReducerContext) {
    // Clean up client state, mark offline, etc.
}
```

## Error Handling

### Return Errors
```rust
#[reducer]
pub fn my_reducer(ctx: &ReducerContext) -> Result<(), String> {
    let row = ctx.db.my_table().id().find(&1)
        .ok_or("Row not found")?;

    if !valid_condition {
        return Err("Invalid condition".to_string());
    }

    Ok(())
}
```

### try_insert vs insert
```rust
// try_insert returns Result - use for graceful handling
match ctx.db.my_table().try_insert(row) {
    Ok(_) => { /* success */ }
    Err(e) => { /* handle error */ }
}

// insert panics on error - use when failure is unexpected
ctx.db.my_table().insert(row);
```
