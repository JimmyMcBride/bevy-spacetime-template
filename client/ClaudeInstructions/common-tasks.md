# Client Common Tasks

## Add a New ECS Component

### 1. Define in components.rs
```rust
use bevy::prelude::*;

/// Description of what this component represents
#[derive(Component)]
pub struct MyNewComponent {
    pub field1: f32,
    pub field2: String,
}

// Or for marker components (no data)
#[derive(Component)]
pub struct MyMarker;
```

### 2. Use in Systems
```rust
// Query entities with the component
fn my_system(query: Query<&MyNewComponent>) {
    for component in query.iter() {
        println!("{}", component.field1);
    }
}

// Add to entity
fn spawn_system(mut commands: Commands) {
    commands.spawn((
        Transform::default(),
        MyNewComponent {
            field1: 1.0,
            field2: "test".to_string(),
        },
    ));
}
```

## Add a New System

### 1. Create System Function
```rust
// In appropriate systems/*.rs file

/// Brief description
pub fn my_new_system(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<LocalPlayer>>,
) {
    for mut transform in query.iter_mut() {
        // Implementation
    }
}
```

### 2. Register in main.rs
```rust
.add_systems(Update, (
    // ... existing systems
    systems::my_module::my_new_system,
))
```

### 3. Export from mod.rs (if new file)
```rust
// systems/mod.rs
pub mod my_module;
```

## Add Input Handling

### Keyboard Input
```rust
pub fn handle_my_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    connection: Res<Connection>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        // Action on press
        let _ = connection.0.reducers.my_action();
    }

    if keyboard.pressed(KeyCode::KeyW) {
        // Continuous action while held
    }
}
```

### With Throttling
```rust
#[derive(Resource)]
pub struct MyInputThrottle {
    timer: Timer,
}

impl Default for MyInputThrottle {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.1, TimerMode::Repeating),
        }
    }
}

pub fn handle_throttled_input(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut throttle: ResMut<MyInputThrottle>,
    connection: Res<Connection>,
) {
    throttle.timer.tick(time.delta());
    if !throttle.timer.just_finished() {
        return;
    }

    // Process input at throttled rate
}
```

## Add Visual Elements

### Spawn 3D Mesh
```rust
fn spawn_object(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Cube
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb(1.0, 0.0, 0.0))),
        Transform::from_xyz(0.0, 0.5, 0.0),
    ));

    // Sphere
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(0.5))),
        MeshMaterial3d(materials.add(Color::srgb(0.0, 1.0, 0.0))),
        Transform::from_xyz(2.0, 0.5, 0.0),
    ));

    // Capsule
    commands.spawn((
        Mesh3d(meshes.add(Capsule3d::new(0.3, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.0, 0.0, 1.0))),
        Transform::from_xyz(-2.0, 0.5, 0.0),
    ));
}
```

### Add Lighting
```rust
// Directional light (sun-like)
commands.spawn((
    DirectionalLight {
        illuminance: 10_000.0,
        shadows_enabled: true,
        ..default()
    },
    Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
));

// Point light
commands.spawn((
    PointLight {
        intensity: 1_000_000.0,
        range: 10.0,
        shadows_enabled: true,
        ..default()
    },
    Transform::from_xyz(0.0, 5.0, 0.0),
));

// Ambient light
commands.insert_resource(AmbientLight {
    color: Color::WHITE,
    brightness: 0.3,
});
```

## Troubleshoot Network Issues

### Check Connection
```rust
fn debug_connection(connection: Res<Connection>) {
    if let Some(identity) = connection.0.try_identity() {
        println!("Connected as: {:?}", identity);
    } else {
        println!("Not connected yet");
    }
}
```

### Check Table Data
```rust
fn debug_tables(connection: Res<Connection>) {
    let count = connection.0.db.entity_position().iter().count();
    println!("entity_position rows: {}", count);

    for pos in connection.0.db.entity_position().iter() {
        println!("  Owner: {:?}, Pos: ({}, {})", pos.owner, pos.x, pos.z);
    }
}
```

### Verify Polling
```rust
// Make sure this system is registered and running
pub fn poll_connection(connection: Res<Connection>) {
    connection.0.frame_tick();
    // Add debug log to verify it's being called
    // println!("Polled connection");
}
```

## Add Camera Shake

```rust
#[derive(Resource, Default)]
pub struct CameraShake {
    pub trauma: f32,  // 0.0 to 1.0
}

pub fn apply_camera_shake(
    time: Res<Time>,
    mut shake: ResMut<CameraShake>,
    mut camera_query: Query<&mut Transform, With<Camera3d>>,
) {
    let Ok(mut camera) = camera_query.get_single_mut() else { return };

    if shake.trauma > 0.0 {
        let shake_amount = shake.trauma * shake.trauma;
        let offset_x = (time.elapsed_secs() * 50.0).sin() * shake_amount * 0.5;
        let offset_y = (time.elapsed_secs() * 60.0).cos() * shake_amount * 0.3;

        camera.translation.x += offset_x;
        camera.translation.y += offset_y;

        // Decay trauma
        shake.trauma = (shake.trauma - time.delta_secs() * 2.0).max(0.0);
    }
}

// Trigger shake from another system
fn on_damage(mut shake: ResMut<CameraShake>) {
    shake.trauma = 1.0;
}
```

## Different Player Colors

```rust
pub fn sync_entities_from_db(
    // ... existing params
) {
    // ... existing logic

    for pos in &db_positions {
        // ... existing find logic

        if existing.is_none() {
            // Determine color based on whether this is local player
            let color = if Some(pos.owner) == local_identity {
                Color::srgb(0.2, 0.8, 0.2)  // Green for local
            } else {
                Color::srgb(0.8, 0.2, 0.2)  // Red for others
            };

            commands.spawn((
                Mesh3d(meshes.add(Capsule3d::new(0.3, 1.0))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: color,
                    ..default()
                })),
                // ... rest of components
            ));
        }
    }
}
```

## Add Player Name Display

```rust
use bevy::text::Text;

#[derive(Component)]
pub struct PlayerNameTag;

fn spawn_player_with_name(
    commands: &mut Commands,
    name: &str,
) {
    commands.spawn((
        // Player entity
        PlayerBundle { ... },
    )).with_children(|parent| {
        // Name tag as child
        parent.spawn((
            Text::new(name),
            TextFont {
                font_size: 16.0,
                ..default()
            },
            Transform::from_xyz(0.0, 1.5, 0.0),
            PlayerNameTag,
        ));
    });
}

// Make name tags face camera (billboard effect)
fn billboard_name_tags(
    camera_query: Query<&Transform, With<Camera3d>>,
    mut tag_query: Query<&mut Transform, (With<PlayerNameTag>, Without<Camera3d>)>,
) {
    let Ok(camera) = camera_query.get_single() else { return };

    for mut tag_transform in tag_query.iter_mut() {
        tag_transform.look_at(camera.translation, Vec3::Y);
    }
}
```
