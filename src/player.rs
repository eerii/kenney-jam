use bevy::prelude::*;

use crate::{
    assets::SpriteAssets,
    input::{Action, ActionState},
    tilemap::TILE_SEP,
    GameState, SCALE,
};

// ······
// Plugin
// ······

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::Play),
            init.run_if(run_once()),
        )
        .add_systems(
            Update,
            update_player.run_if(in_state(GameState::Play)),
        );
    }
}

// ··········
// Components
// ··········

#[derive(Component)]
struct Player;

// ·······
// Systems
// ·······

fn init(mut cmd: Commands, sprite_assets: Res<SpriteAssets>) {
    cmd.spawn((
        SpriteBundle {
            transform: Transform::from_xyz(0., 0., 10.).with_scale(Vec3::splat(SCALE)),
            texture: sprite_assets.one_bit.clone(),
            ..default()
        },
        TextureAtlas {
            layout: sprite_assets.one_bit_atlas.clone(),
            index: 25,
        },
        Player,
    ));
}

fn update_player(
    input: Query<&ActionState<Action>>,
    mut player: Query<&mut Transform, With<Player>>,
) {
    let Ok(mut trans) = player.get_single_mut() else { return };
    let Ok(input) = input.get_single() else { return };

    if input.just_pressed(&Action::Move) {
        let Some(axis) = input.clamped_axis_pair(&Action::Move) else { return };
        if axis.x().abs() > axis.y().abs() {
            trans.translation.x += axis.x().signum() * TILE_SEP * SCALE;
        } else {
            trans.translation.y += axis.y().signum() * TILE_SEP * SCALE;
        }
    };
}
