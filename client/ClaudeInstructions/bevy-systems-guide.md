# Bevy Systems Guide

## System Function Signatures

### Basic System
```rust
fn my_system() {
    // No parameters - runs every frame
}
```

### With Query
```rust
fn my_system(query: Query<&Transform>) {
    for transform in query.iter() {
        println!("Position: {:?}", transform.translation);
    }
}
```

### With Mutable Query
```rust
fn my_system(mut query: Query<&mut Transform>) {
    for mut transform in query.iter_mut() {
        transform.translation.y += 1.0;
    }
}
```

### With Resources
```rust
fn my_system(time: Res<Time>, mut throttle: ResMut<InputThrottle>) {
    throttle.timer.tick(time.delta());
}
```

### With Commands
```rust
fn my_system(mut commands: Commands) {
    commands.spawn((
        Transform::default(),
        Visibility::default(),
    ));
}
```

## Query Patterns

### Basic Query
```rust
Query<&Transform>                    // Read-only Transform
Query<&mut Transform>                // Mutable Transform
Query<(&Transform, &Health)>         // Multiple components
Query<(Entity, &Transform)>          // Include Entity ID
```

### With Filters
```rust
// Only entities WITH LocalPlayer component
Query<&Transform, With<LocalPlayer>>

// Only entities WITHOUT Camera3d component
Query<&Transform, Without<Camera3d>>

// Combined filters
Query<&Transform, (With<LocalPlayer>, Without<Dead>)>

// Changed components (since last frame)
Query<&Transform, Changed<Transform>>

// Added components (this frame)
Query<&NetworkPlayer, Added<NetworkPlayer>>
```

### Query Methods
```rust
fn my_system(query: Query<&Transform, With<LocalPlayer>>) {
    // Iterate all
    for transform in query.iter() { }

    // Iterate mutable
    for mut transform in query.iter_mut() { }

    // Get single (panics if not exactly one)
    let transform = query.single();

    // Get single (returns Result)
    let Ok(transform) = query.get_single() else { return };

    // Get by entity ID
    if let Ok(transform) = query.get(entity_id) { }

    // Check if empty
    if query.is_empty() { return; }

    // Count
    let count = query.iter().count();
}
```

## Resource Access

### Read-Only Resource
```rust
fn my_system(time: Res<Time>) {
    let delta = time.delta_secs();
}
```

### Mutable Resource
```rust
fn my_system(mut throttle: ResMut<InputThrottle>) {
    throttle.timer.tick(time.delta());
}
```

### Optional Resource
```rust
fn my_system(connection: Option<Res<Connection>>) {
    let Some(conn) = connection else { return };
    // Use conn
}
```

### Insert Resource (from system)
```rust
fn my_system(mut commands: Commands) {
    commands.insert_resource(MyResource::new());
}
```

## System Scheduling

### Startup vs Update
```rust
App::new()
    .add_systems(Startup, setup_system)    // Runs once at start
    .add_systems(Update, update_system)    // Runs every frame
```

### System Ordering
```rust
// Run in specific order
.add_systems(Update, (
    system_a,
    system_b.after(system_a),
    system_c.before(system_a),
))

// Chain systems (A then B then C)
.add_systems(Update, (system_a, system_b, system_c).chain())
```

### Conditional Systems
```rust
// Only run if resource exists
.add_systems(Update, my_system.run_if(resource_exists::<Connection>))

// Only run if condition is true
.add_systems(Update, my_system.run_if(|query: Query<&LocalPlayer>| !query.is_empty()))
```

## Commands for Entity Manipulation

### Spawn Entity
```rust
fn spawn_player(mut commands: Commands) {
    commands.spawn((
        Transform::from_xyz(0.0, 1.0, 0.0),
        Visibility::default(),
        PlayerMarker,
    ));
}
```

### Spawn with Mesh
```rust
fn spawn_visual(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Capsule3d::new(0.3, 1.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.6, 0.8),
            ..default()
        })),
        Transform::from_xyz(0.0, 0.5, 0.0),
    ));
}
```

### Despawn Entity
```rust
fn despawn_dead(
    mut commands: Commands,
    query: Query<Entity, With<Dead>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
```

### Add Component to Entity
```rust
fn add_marker(
    mut commands: Commands,
    query: Query<Entity, (With<NetworkPlayer>, Without<Processed>)>,
) {
    for entity in query.iter() {
        commands.entity(entity).insert(Processed);
    }
}
```

### Remove Component
```rust
commands.entity(entity).remove::<MyComponent>();
```

## Common Resources

### Time
```rust
fn my_system(time: Res<Time>) {
    let delta = time.delta_secs();           // Seconds since last frame
    let elapsed = time.elapsed_secs();       // Total seconds
}
```

### Input
```rust
fn my_system(keyboard: Res<ButtonInput<KeyCode>>) {
    if keyboard.pressed(KeyCode::KeyW) { }   // Held down
    if keyboard.just_pressed(KeyCode::Space) { }  // Just pressed this frame
    if keyboard.just_released(KeyCode::Space) { } // Just released
}
```

### Assets
```rust
fn my_system(
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh_handle = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let material_handle = materials.add(Color::srgb(1.0, 0.0, 0.0));
}
```

## Bevy 0.15 Specific Patterns

### 3D Mesh Components
```rust
// Bevy 0.15 uses Mesh3d and MeshMaterial3d
commands.spawn((
    Mesh3d(meshes.add(Capsule3d::new(0.3, 1.0))),
    MeshMaterial3d(materials.add(StandardMaterial {
        base_color: Color::srgb(0.2, 0.6, 0.8),
        ..default()
    })),
    Transform::from_xyz(0.0, 0.5, 0.0),
));
```

### Camera Setup
```rust
commands.spawn((
    Camera3d::default(),
    Projection::from(OrthographicProjection {
        scaling_mode: ScalingMode::FixedVertical { viewport_height: 10.0 },
        ..OrthographicProjection::default_3d()
    }),
    Transform::from_xyz(10.0, 10.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
));
```

### Directional Light
```rust
commands.spawn((
    DirectionalLight {
        illuminance: 10_000.0,
        shadows_enabled: true,
        ..default()
    },
    Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
));
```
