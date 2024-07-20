use bevy::prelude::*;
use itertools::Itertools;

use crate::{
    assets::{SpriteAssets, ATLAS_SIZE},
    data::{Persistent, SaveData},
    player::{Status, StatusEvent},
    GameState, PlaySet, PlayState, SCALE,
};

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
    // TODO: Convert to hashset or something more sparse
    tiles: Vec<Entity>,
    size: UVec2,
}

impl Tilemap {
    pub fn get_tile(&self, pos: UVec2) -> Option<Entity> {
        if pos.y >= self.size.y {
            return None;
        };
        let i = (pos.x * self.size.y + pos.y) as usize;
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
    LadderDown,
    LadderUp,
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
    let ladder_up = (size.x / 2) * size.y + (size.y / 2);
    let mut ladder_down = rand::random::<u32>() % (size.x * size.y - 1);
    if ladder_down == ladder_up {
        ladder_down = size.x * size.y - 1;
    }

    for (x, y) in (0..size.x).cartesian_product(0..size.y) {
        let i = x * size.y + y;
        let (tile, index) = if i == ladder_down {
            (
                TileType::LadderDown,
                6 * ATLAS_SIZE.0 + 3,
            )
        } else if i == ladder_up {
            (TileType::LadderUp, 6 * ATLAS_SIZE.0 + 2)
        } else {
            let index = rand::random::<usize>() % 9;
            let tile = if index == 8 { TileType::Collision } else { TileType::default() };
            (tile, index)
        };

        let tile = cmd.spawn((
            SpriteBundle {
                transform: Transform::from_translation(tile_to_pos(UVec2::new(x, y)).extend(0.))
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
    mut save_data: ResMut<Persistent<SaveData>>,
    mut status_writer: EventWriter<StatusEvent>,
) {
    next_state.set(GameState::Play);
    next_play_state.set(PlayState::Play);
    let _ = save_data.update(|data| data.level += 1);
    if save_data.level >= save_data.max_range {
        status_writer.send(StatusEvent(Status::ConnectionEmpty));
    } else if save_data.level + 2 >= save_data.max_range {
        status_writer.send(StatusEvent(Status::ConnectionLow));
    }
}

// ·······
// Helpers
// ·······

pub fn pos_to_tile(pos: Vec2) -> UVec2 {
    let pos = pos / TILE_SEP / SCALE;
    UVec2::new((pos.x + 5.) as u32, (pos.y + 3.) as u32)
}

pub fn tile_to_pos(pos: UVec2) -> Vec2 {
    Vec2::new(
        (pos.x as f32 - 5.) * TILE_SEP * SCALE,
        (pos.y as f32 - 3.) * TILE_SEP * SCALE,
    )
}
