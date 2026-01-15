use spacetimedb::{table, reducer, ReducerContext, Identity, Table, ScheduleAt};
use std::time::Duration;

/// Player table - stores persistent player data
#[table(name = player, public)]
#[derive(Clone)]
pub struct Player {
    #[primary_key]
    pub identity: Identity,
    pub name: String,
    pub online: bool,
}

/// Entity position table - stores real-time position data
#[table(name = entity_position, public)]
#[derive(Clone)]
pub struct EntityPosition {
    #[primary_key]
    pub owner: Identity,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub direction_x: f32,
    pub direction_z: f32,
    pub moving: bool,
}

/// Timer table for movement ticks - runs tick_movement at 50ms intervals
#[table(name = movement_timer, scheduled(tick_movement))]
pub struct MovementTimer {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub scheduled_at: ScheduleAt,
}

/// Called when client first connects to create their player
#[reducer]
pub fn create_player(ctx: &ReducerContext, name: String) -> Result<(), String> {
    let identity = ctx.sender;

    // Check if player already exists
    if ctx.db.player().identity().find(identity).is_some() {
        return Ok(());
    }

    // Create player record
    ctx.db.player().try_insert(Player {
        identity,
        name,
        online: true,
    })?;

    // Create initial position at origin
    ctx.db.entity_position().try_insert(EntityPosition {
        owner: identity,
        x: 0.0,
        y: 0.0,
        z: 0.0,
        direction_x: 0.0,
        direction_z: 0.0,
        moving: false,
    })?;

    Ok(())
}

/// Update player movement input - called from client when movement keys pressed
#[reducer]
pub fn update_player_input(
    ctx: &ReducerContext,
    direction_x: f32,
    direction_z: f32,
    moving: bool,
) -> Result<(), String> {
    let identity = ctx.sender;

    // Get player's position
    let mut position = ctx.db.entity_position().owner().find(identity)
        .ok_or("Player position not found")?;

    // Normalize direction vector
    let length = (direction_x * direction_x + direction_z * direction_z).sqrt();
    if length > 0.0 {
        position.direction_x = direction_x / length;
        position.direction_z = direction_z / length;
    } else {
        position.direction_x = 0.0;
        position.direction_z = 0.0;
    }

    position.moving = moving;

    ctx.db.entity_position().owner().update(position);
    Ok(())
}

/// Initialize the movement timer - runs once when module is published
#[reducer(init)]
pub fn init(ctx: &ReducerContext) {
    start_movement_timer_impl(ctx);
}

/// Manually start the movement timer (call this after updating an existing database)
#[reducer]
pub fn start_movement_timer(ctx: &ReducerContext) -> Result<(), String> {
    start_movement_timer_impl(ctx);
    Ok(())
}

fn start_movement_timer_impl(ctx: &ReducerContext) {
    // Only insert if no timer exists
    if ctx.db.movement_timer().iter().next().is_none() {
        ctx.db.movement_timer().try_insert(MovementTimer {
            id: 0,
            scheduled_at: Duration::from_millis(50).into(),
        }).ok();
    }
}

/// Physics tick - runs server-side at 20Hz (every 50ms)
/// Moves all active players based on their direction and speed
#[reducer]
pub fn tick_movement(ctx: &ReducerContext, _timer: MovementTimer) -> Result<(), String> {
    const SPEED: f32 = 0.05;
    const DELTA_TIME: f32 = 0.05; // 50ms tick rate

    // Update all moving entities
    for mut entity in ctx.db.entity_position().iter() {
        if entity.moving {
            entity.x += entity.direction_x * SPEED * DELTA_TIME;
            entity.z += entity.direction_z * SPEED * DELTA_TIME;

            // Enforce world bounds (±50 units)
            entity.x = entity.x.clamp(-50.0, 50.0);
            entity.z = entity.z.clamp(-50.0, 50.0);

            ctx.db.entity_position().owner().update(entity);
        }
    }

    // Re-schedule the next tick (the current timer row is automatically deleted)
    ctx.db.movement_timer().try_insert(MovementTimer {
        id: 0,
        scheduled_at: Duration::from_millis(50).into(),
    }).ok();

    Ok(())
}

/// Called when client connects
#[reducer(client_connected)]
pub fn on_client_connected(ctx: &ReducerContext) {
    // Set player online status
    if let Some(mut player) = ctx.db.player().identity().find(ctx.sender) {
        player.online = true;
        ctx.db.player().identity().update(player);
    }
}

/// Called when client disconnects
#[reducer(client_disconnected)]
pub fn on_client_disconnected(ctx: &ReducerContext) {
    // Set player offline status
    if let Some(mut player) = ctx.db.player().identity().find(ctx.sender) {
        player.online = false;
        ctx.db.player().identity().update(player);
    }

    // Delete the player's position (removes their capsule for other clients)
    ctx.db.entity_position().owner().delete(ctx.sender);
}
