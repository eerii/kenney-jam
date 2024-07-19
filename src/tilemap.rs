use bevy::prelude::*;
use itertools::Itertools;

use crate::{assets::SpriteAssets, GameState, SCALE};

pub const TILE_SEP: f32 = 20.;

// ······
// Plugin
// ······

pub struct TilemapPlugin;

impl Plugin for TilemapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::Play),
            init.run_if(run_once()),
        );
    }
}

// ··········
// Components
// ··········

#[derive(Component)]
struct Tile;

// ·······
// Systems
// ·······

fn init(mut cmd: Commands, sprite_assets: Res<SpriteAssets>) {
    for (x, y) in (-5..=5).cartesian_product(-3..=3) {
        cmd.spawn((
            SpriteBundle {
                transform: Transform::from_xyz(
                    x as f32 * TILE_SEP * SCALE,
                    y as f32 * TILE_SEP * SCALE,
                    0.,
                )
                .with_scale(Vec3::splat(SCALE)),
                texture: sprite_assets.one_bit.clone(),
                ..default()
            },
            TextureAtlas {
                layout: sprite_assets.one_bit_atlas.clone(),
                index: rand::random::<usize>() % 8,
            },
            Tile,
        ));
    }
}
