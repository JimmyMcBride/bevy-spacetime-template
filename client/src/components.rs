use bevy::prelude::*;
use spacetimedb_sdk::Identity;

/// Marks the local player controlled by this client
#[derive(Component)]
pub struct LocalPlayer;

/// Network player with SpacetimeDB identity
#[derive(Component)]
pub struct NetworkPlayer {
    pub identity: Identity,
}

/// Smooth position interpolation data
#[derive(Component)]
pub struct PositionInterpolation {
    pub target_position: Vec3,
    pub speed: f32,
}

/// Marker for player visual mesh
#[derive(Component)]
pub struct PlayerCapsule;
