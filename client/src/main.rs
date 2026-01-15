mod components;
mod systems;
mod module_bindings;

use bevy::prelude::*;
use systems::*;

#[derive(Resource)]
pub struct Connection(pub module_bindings::DbConnection);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(create_connection())
        .insert_resource(input::InputThrottle::default())
        .add_systems(Startup, (
            camera::setup_camera,
            setup_scene,
            network::setup_connection,
        ))
        .add_systems(Update, (
            network::poll_connection,
            network::sync_entities_from_db,
            input::handle_player_input,
            movement::interpolate_positions,
            camera::follow_local_player,
        ))
        .run();
}

fn create_connection() -> Connection {
    let connection = module_bindings::DbConnection::builder()
        .with_uri("https://maincloud.spacetimedb.com")
        .with_module_name("YOUR_DATABASE_NAME")  // <-- Change this!
        .build()
        .expect("Failed to connect to SpacetimeDB");
    Connection(connection)
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Ground plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(50.0)))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.5, 0.3),
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    // Lighting
    commands.spawn((
        DirectionalLight {
            illuminance: 10_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.3,
    });
}
