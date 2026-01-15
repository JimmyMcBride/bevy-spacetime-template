use bevy::prelude::*;
use spacetimedb_sdk::{DbContext, Table};
use crate::components::*;
use crate::module_bindings::{DbConnection, EntityPosition, entity_position_table::EntityPositionTableAccess};
use crate::Connection;

/// Set up subscriptions and create player
pub fn setup_connection(connection: Res<Connection>) {
    let conn = &connection.0;

    conn.subscription_builder()
        .subscribe(["SELECT * FROM player"]);

    conn.subscription_builder()
        .subscribe(["SELECT * FROM entity_position"]);

    let _ = conn.reducers.create_player("Player".to_string());
}

/// Poll for network updates (call every frame)
pub fn poll_connection(connection: Res<Connection>) {
    connection.0.frame_tick();
}

/// Sync database state to Bevy entities
pub fn sync_entities_from_db(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    connection: Res<Connection>,
    mut query: Query<(Entity, &NetworkPlayer, &mut PositionInterpolation)>,
) {
    let local_identity = connection.0.try_identity();
    let db_positions: Vec<EntityPosition> = connection.0.db.entity_position().iter().collect();

    // Track which DB entities we've seen
    let mut seen_identities = std::collections::HashSet::new();

    for pos in &db_positions {
        seen_identities.insert(pos.owner);

        // Find existing entity or spawn new one
        let existing = query.iter_mut().find(|(_, np, _)| np.identity == pos.owner);

        if let Some((entity, _, mut interp)) = existing {
            // Update target position
            interp.target_position = Vec3::new(pos.x, 0.5, pos.z);
        } else {
            // Spawn new player capsule
            let mut entity_commands = commands.spawn((
                Mesh3d(meshes.add(Capsule3d::new(0.3, 1.0))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgb(0.2, 0.6, 0.8),
                    ..default()
                })),
                Transform::from_xyz(pos.x, 0.5, pos.z),
                NetworkPlayer { identity: pos.owner },
                PositionInterpolation {
                    target_position: Vec3::new(pos.x, 0.5, pos.z),
                    speed: 10.0,
                },
                PlayerCapsule,
            ));

            // Mark local player
            if Some(pos.owner) == local_identity {
                entity_commands.insert(LocalPlayer);
            }
        }
    }

    // Remove entities no longer in database
    for (entity, np, _) in query.iter() {
        if !seen_identities.contains(&np.identity) {
            commands.entity(entity).despawn_recursive();
        }
    }
}
