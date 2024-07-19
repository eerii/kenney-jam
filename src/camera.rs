//! Camera module

use bevy::prelude::*;

use crate::{
    data::{init_data, GameOptions, Persistent},
    player::Player,
    GameState, PlaySet,
};

/// The luminance of the background color
pub const BACKGROUND_LUMINANCE: f32 = 0.05;

// ······
// Plugin
// ······

/// Camera
/// Creates the main game camera, marked by `GameCamera`
/// Depending on the 3d_camera feature it will be 2d or 3d
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::Startup),
            init.after(init_data),
        )
        .add_systems(
            Update,
            update_camera.after(PlaySet::Animation),
        );
    }
}

// ··········
// Components
// ··········

/// The camera where the game is being rendered
#[derive(Component)]
pub struct GameCamera;

/// The camera that renders everything to the screen
/// It can be different from the GameCamera if doing any kind of
/// deferred rendering or pixel scaling
#[derive(Component)]
pub struct FinalCamera;

// ·······
// Systems
// ·······

/// Creates the main cameras before the game starts
fn init(mut cmd: Commands, options: Res<Persistent<GameOptions>>) {
    let clear_color =
        ClearColorConfig::Custom(options.base_color.with_luminance(BACKGROUND_LUMINANCE));

    let camera_bundle = Camera2dBundle {
        camera: Camera {
            clear_color,
            ..default()
        },
        ..default()
    };

    cmd.spawn((camera_bundle, GameCamera, FinalCamera));
}

fn update_camera(
    player: Query<&Transform, (With<Player>, Without<GameCamera>)>,
    mut cam: Query<&mut Transform, With<GameCamera>>,
) {
    let Ok(player) = player.get_single() else { return };
    let Ok(mut trans) = cam.get_single_mut() else { return };

    let target_pos = player.translation.truncate();
    let pos = trans.translation.truncate().lerp(target_pos, 0.1);
    trans.translation = pos.extend(trans.translation.z);
}
