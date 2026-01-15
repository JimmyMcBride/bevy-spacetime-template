use spacetimedb::{table, reducer, ReducerContext, Identity, Table, ScheduleAt};
use std::time::Duration;

// ============================================================================
// TABLES
// ============================================================================

/// Player identity and metadata
#[table(name = player, public)]
#[derive(Clone)]
pub struct Player {
    #[primary_key]
    pub identity: Identity,
    pub name: String,
    pub online: bool,
}

/// Real-time entity position (server-authoritative)
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

/// Scheduled table for physics tick loop
#[table(name = movement_timer, scheduled(tick_movement))]
pub struct MovementTimer {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub scheduled_at: ScheduleAt,
}

// ============================================================================
// REDUCERS
// ============================================================================

/// Initialize movement timer on first publish
#[reducer(init)]
pub fn init(ctx: &ReducerContext) {
    start_movement_timer_impl(ctx);
}

/// Manually start timer (call after updating existing database)
#[reducer]
pub fn start_movement_timer(ctx: &ReducerContext) -> Result<(), String> {
    start_movement_timer_impl(ctx);
    Ok(())
}

fn start_movement_timer_impl(ctx: &ReducerContext) {
    if ctx.db.movement_timer().iter().next().is_none() {
        ctx.db.movement_timer().try_insert(MovementTimer {
            id: 0,
            scheduled_at: Duration::from_millis(50).into(),
        }).ok();
    }
}

/// Create new player (called by client on connect)
#[reducer]
pub fn create_player(ctx: &ReducerContext, name: String) -> Result<(), String> {
    let identity = ctx.sender;

    if ctx.db.player().identity().find(&identity).is_some() {
        return Ok(()); // Already exists
    }

    ctx.db.player().try_insert(Player {
        identity,
        name,
        online: true,
    })?;

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

/// Update player input (called by client on WASD press)
#[reducer]
pub fn update_player_input(
    ctx: &ReducerContext,
    direction_x: f32,
    direction_z: f32,
    moving: bool,
) -> Result<(), String> {
    let identity = ctx.sender;

    let mut position = ctx.db.entity_position().owner().find(&identity)
        .ok_or("Player position not found")?;

    // Normalize direction
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

/// Physics tick - runs at 20Hz (50ms intervals)
#[reducer]
pub fn tick_movement(ctx: &ReducerContext, _timer: MovementTimer) -> Result<(), String> {
    const SPEED: f32 = 5.0;        // Units per second
    const DELTA_TIME: f32 = 0.05;  // 50ms
    const WORLD_BOUNDS: f32 = 50.0;

    for mut entity in ctx.db.entity_position().iter() {
        if entity.moving {
            entity.x += entity.direction_x * SPEED * DELTA_TIME;
            entity.z += entity.direction_z * SPEED * DELTA_TIME;

            // Clamp to world bounds
            entity.x = entity.x.clamp(-WORLD_BOUNDS, WORLD_BOUNDS);
            entity.z = entity.z.clamp(-WORLD_BOUNDS, WORLD_BOUNDS);

            ctx.db.entity_position().owner().update(entity);
        }
    }

    // Re-schedule next tick
    ctx.db.movement_timer().try_insert(MovementTimer {
        id: 0,
        scheduled_at: Duration::from_millis(50).into(),
    }).ok();

    Ok(())
}

/// Lifecycle: client connected
#[reducer(client_connected)]
pub fn on_client_connected(ctx: &ReducerContext) {
    if let Some(mut player) = ctx.db.player().identity().find(&ctx.sender) {
        player.online = true;
        ctx.db.player().identity().update(player);
    }
}

/// Lifecycle: client disconnected
#[reducer(client_disconnected)]
pub fn on_client_disconnected(ctx: &ReducerContext) {
    if let Some(mut player) = ctx.db.player().identity().find(&ctx.sender) {
        player.online = false;
        ctx.db.player().identity().update(player);
    }
}
