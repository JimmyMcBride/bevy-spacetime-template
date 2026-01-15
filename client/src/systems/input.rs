use bevy::prelude::*;
use crate::components::LocalPlayer;
use crate::Connection;

#[derive(Resource)]
pub struct InputThrottle {
    timer: Timer,
}

impl Default for InputThrottle {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.05, TimerMode::Repeating), // 20Hz
        }
    }
}

pub fn handle_player_input(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut throttle: ResMut<InputThrottle>,
    connection: Res<Connection>,
    player_query: Query<&LocalPlayer>,
) {
    // Only process if we have a local player
    if player_query.is_empty() {
        return;
    }

    throttle.timer.tick(time.delta());
    if !throttle.timer.just_finished() {
        return;
    }

    // Calculate direction from input
    let mut direction = Vec3::ZERO;

    if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
        direction.z -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
        direction.z += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
        direction.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        direction.x += 1.0;
    }

    let moving = direction.length() > 0.0;

    let _ = connection.0.reducers.update_player_input(
        direction.x,
        direction.z,
        moving,
    );
}
