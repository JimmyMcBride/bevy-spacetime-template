# Server Common Tasks

## Add a New Table

### 1. Define the Table
```rust
/// Description of what this table stores
#[table(name = my_new_table, public)]
#[derive(Clone)]
pub struct MyNewTable {
    #[primary_key]
    pub id: u64,  // or Identity for player-owned
    pub field1: String,
    pub field2: f32,
}
```

### 2. Add Initialization (if needed)
```rust
#[reducer(init)]
pub fn init(ctx: &ReducerContext) {
    // Initialize default rows if needed
    ctx.db.my_new_table().try_insert(MyNewTable {
        id: 0,
        field1: "default".to_string(),
        field2: 0.0,
    }).ok();
}
```

### 3. Rebuild and Regenerate Bindings
```bash
cd server
cargo build --target=wasm32-unknown-unknown --release
spacetime publish YOUR_DB_NAME --server maincloud
spacetime generate --lang rust --out-dir ../client/src/module_bindings --project-path .
```

## Add a New Reducer

### 1. Define the Reducer
```rust
/// Description of what this reducer does
#[reducer]
pub fn my_new_reducer(
    ctx: &ReducerContext,
    arg1: String,
    arg2: i32,
) -> Result<(), String> {
    // Validate input
    if arg1.is_empty() {
        return Err("arg1 cannot be empty".to_string());
    }

    // Get player
    let identity = ctx.sender;

    // Do work
    ctx.db.my_table().try_insert(/* ... */)?;

    Ok(())
}
```

### 2. Regenerate Client Bindings
```bash
cd server
spacetime generate --lang rust --out-dir ../client/src/module_bindings --project-path .
```

### 3. Call from Client
```rust
// In client code
connection.reducers.my_new_reducer("value".to_string(), 42);
```

## Add a Scheduled Task

### 1. Create Timer Table
```rust
#[table(name = my_task_timer, scheduled(run_my_task))]
pub struct MyTaskTimer {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub scheduled_at: ScheduleAt,
}
```

### 2. Create Task Reducer
```rust
#[reducer]
pub fn run_my_task(ctx: &ReducerContext, _timer: MyTaskTimer) -> Result<(), String> {
    // Do periodic work here

    // Re-schedule (for recurring task)
    ctx.db.my_task_timer().try_insert(MyTaskTimer {
        id: 0,
        scheduled_at: Duration::from_secs(60).into(),  // Every 60 seconds
    }).ok();

    Ok(())
}
```

### 3. Start Timer in Init
```rust
#[reducer(init)]
pub fn init(ctx: &ReducerContext) {
    // Start the task timer
    ctx.db.my_task_timer().try_insert(MyTaskTimer {
        id: 0,
        scheduled_at: Duration::from_secs(1).into(),  // First run in 1 second
    }).ok();
}
```

## Update an Existing Schema

### Adding a New Field
```rust
// Old
pub struct Player {
    #[primary_key]
    pub identity: Identity,
    pub name: String,
}

// New - add field with default-able type
pub struct Player {
    #[primary_key]
    pub identity: Identity,
    pub name: String,
    pub score: u32,  // New field - will be 0 for existing rows
}
```

### Breaking Changes
For breaking changes (removing fields, changing types), you may need to:
1. Delete the database: `spacetime delete YOUR_DB_NAME --server maincloud`
2. Republish: `spacetime publish YOUR_DB_NAME --server maincloud`

**Warning**: This destroys all existing data.

## Debug with Logs

### Add Logging
```rust
#[reducer]
pub fn my_reducer(ctx: &ReducerContext) -> Result<(), String> {
    // Use println! for logging
    println!("my_reducer called by {:?}", ctx.sender);

    let count = ctx.db.my_table().iter().count();
    println!("Table has {} rows", count);

    Ok(())
}
```

### View Logs
```bash
# View recent logs
spacetime logs YOUR_DB_NAME --server maincloud

# Follow logs in real-time
spacetime logs YOUR_DB_NAME --server maincloud -f
```

## Query the Database

### Using SQL
```bash
# Count players
spacetime sql YOUR_DB_NAME --server maincloud "SELECT COUNT(*) FROM player"

# View all positions
spacetime sql YOUR_DB_NAME --server maincloud "SELECT * FROM entity_position"

# Find specific player
spacetime sql YOUR_DB_NAME --server maincloud "SELECT * FROM player WHERE name = 'Bob'"
```

## Handle Player Disconnect Cleanup

### Mark Player Offline
```rust
#[reducer(client_disconnected)]
pub fn on_client_disconnected(ctx: &ReducerContext) {
    if let Some(mut player) = ctx.db.player().identity().find(&ctx.sender) {
        player.online = false;
        ctx.db.player().identity().update(player);
    }
}
```

### Remove Player Data (Optional)
```rust
#[reducer(client_disconnected)]
pub fn on_client_disconnected(ctx: &ReducerContext) {
    // Remove position data
    ctx.db.entity_position().owner().delete(&ctx.sender);

    // Remove player record
    ctx.db.player().identity().delete(&ctx.sender);
}
```

## Validate Reducer Input

```rust
#[reducer]
pub fn update_player_name(ctx: &ReducerContext, name: String) -> Result<(), String> {
    // Length validation
    if name.len() < 3 {
        return Err("Name must be at least 3 characters".to_string());
    }
    if name.len() > 20 {
        return Err("Name must be at most 20 characters".to_string());
    }

    // Character validation
    if !name.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err("Name can only contain letters, numbers, and underscores".to_string());
    }

    // Update player
    let mut player = ctx.db.player().identity().find(&ctx.sender)
        .ok_or("Player not found")?;
    player.name = name;
    ctx.db.player().identity().update(player);

    Ok(())
}
```
