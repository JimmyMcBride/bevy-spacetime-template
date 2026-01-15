use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use crate::components::LocalPlayer;

const CAMERA_OFFSET: Vec3 = Vec3::new(10.0, 10.0, 10.0);

/// Set up isometric camera
pub fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Projection::from(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical { viewport_height: 10.0 },
            ..OrthographicProjection::default_3d()
        }),
        Transform::from_xyz(10.0, 10.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

/// Smooth camera follow for local player
pub fn follow_local_player(
    time: Res<Time>,
    player_query: Query<&Transform, (With<LocalPlayer>, Without<Camera3d>)>,
    mut camera_query: Query<&mut Transform, With<Camera3d>>,
) {
    let Ok(player_transform) = player_query.get_single() else { return };
    let Ok(mut camera_transform) = camera_query.get_single_mut() else { return };

    let target_pos = player_transform.translation;
    let desired_camera_pos = target_pos + CAMERA_OFFSET;

    // Smooth follow
    let smoothing = 1.0 - (-12.0 * time.delta_secs()).exp();
    camera_transform.translation = camera_transform.translation.lerp(desired_camera_pos, smoothing);

    let look_target = camera_transform.translation - CAMERA_OFFSET;
    *camera_transform = camera_transform.looking_at(look_target, Vec3::Y);
}
