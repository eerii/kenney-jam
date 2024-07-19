use bevy::prelude::*;
use itertools::Itertools;

use crate::{assets::SpriteAssets, GameState, PlaySet, PlayState, SCALE};

pub const TILE_SEP: f32 = 20.;

// ······
// Plugin
// ······

pub struct TilemapPlugin;

impl Plugin for TilemapPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<NextLevelEvent>()
            .add_systems(OnEnter(GameState::Play), init)
            .add_systems(
                Update,
                on_next_level.in_set(PlaySet::Events),
            )
            .add_systems(
                OnEnter(GameState::LevelTransition),
                level_transition,
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
    Ladder,
}

#[derive(Component, Default)]
pub struct Tile {
    _x: u32,
    _y: u32,
    pub tile: TileType,
}

// ······
// Events
// ······

#[derive(Event)]
pub struct NextLevelEvent;

// ·······
// Systems
// ·······

fn init(mut cmd: Commands, sprite_assets: Res<SpriteAssets>) {
    let mut tiles = vec![];
    let size = UVec2::new(11, 7);
    let ladder = rand::random::<u32>() % (size.x * size.y);

    for (x, y) in (0..size.x).cartesian_product(0..size.y) {
        let (tile, index) = if x * size.y + y == ladder {
            (TileType::Ladder, 3 + 6 * 48)
        } else {
            let index = rand::random::<usize>() % 9;
            let tile = if index == 8 { TileType::Collision } else { TileType::default() };
            (tile, index)
        };

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
            Tile { _x: x, _y: y, tile },
            StateScoped(GameState::Play),
        ));
        tiles.push(tile.id());
    }

    cmd.insert_resource(Tilemap { tiles, size });
}

fn on_next_level(
    mut next_state: ResMut<NextState<GameState>>,
    mut next_level_reader: EventReader<NextLevelEvent>,
) {
    if let Some(NextLevelEvent) = next_level_reader.read().next() {
        next_state.set(GameState::LevelTransition);
    }
}

fn level_transition(
    mut next_state: ResMut<NextState<GameState>>,
    mut next_play_state: ResMut<NextState<PlayState>>,
) {
    next_state.set(GameState::Play);
    next_play_state.set(PlayState::Play);
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
