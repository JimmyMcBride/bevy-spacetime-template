use bevy::prelude::*;
use crate::components::PositionInterpolation;

/// Smooth interpolation for 60fps visuals from 20Hz server updates
pub fn interpolate_positions(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &PositionInterpolation)>,
) {
    for (mut transform, interp) in query.iter_mut() {
        let current = transform.translation;
        let target = interp.target_position;

        // Exponential smoothing (framerate-independent)
        let smoothing = 1.0 - (-interp.speed * time.delta_secs()).exp();
        transform.translation = current.lerp(target, smoothing);
    }
}
