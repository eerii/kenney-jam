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

// ·········
// Resources
// ·········

#[derive(Resource)]
pub struct Tilemap {
    tiles: Vec<Entity>,
    size: UVec2,
}

impl Tilemap {
    pub fn get_tile(&self, x: u32, y: u32) -> Option<Entity> {
        if y >= self.size.y {
            return None;
        };
        let i = (x * self.size.y + y) as usize;
        if i < self.tiles.len() {
            Some(self.tiles[i])
        } else {
            None
        }
    }
}

// ··········
// Components
// ··········

#[derive(Default)]
pub enum TileType {
    #[default]
    None,
    Collision,
}

#[derive(Component, Default)]
pub struct Tile {
    _x: u32,
    _y: u32,
    pub tile: TileType,
}

// ·······
// Systems
// ·······

fn init(mut cmd: Commands, sprite_assets: Res<SpriteAssets>) {
    let mut tiles = vec![];
    let size = UVec2::new(11, 7);

    for (x, y) in (0..size.x).cartesian_product(0..size.y) {
        let index = rand::random::<usize>() % 9;

        let tile = cmd.spawn((
            SpriteBundle {
                transform: Transform::from_translation(tile_to_pos(x, y).extend(0.))
                    .with_scale(Vec3::splat(SCALE)),
                texture: sprite_assets.one_bit.clone(),
                ..default()
            },
            TextureAtlas {
                layout: sprite_assets.one_bit_atlas.clone(),
                index,
            },
            Tile {
                _x: x,
                _y: y,
                tile: if index == 8 { TileType::Collision } else { TileType::default() },
            },
        ));
        tiles.push(tile.id());
    }

    cmd.insert_resource(Tilemap { tiles, size });
}

// ·······
// Helpers
// ·······

pub fn pos_to_tile(pos: Vec2) -> (u32, u32) {
    let pos = pos / TILE_SEP / SCALE;
    ((pos.x + 5.) as u32, (pos.y + 3.) as u32)
}

pub fn tile_to_pos(x: u32, y: u32) -> Vec2 {
    Vec2::new(
        (x as f32 - 5.) * TILE_SEP * SCALE,
        (y as f32 - 3.) * TILE_SEP * SCALE,
    )
}
