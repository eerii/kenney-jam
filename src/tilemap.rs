use std::{
    collections::{HashMap, HashSet},
    hash::{Hash, Hasher},
};

use bevy::prelude::*;
use itertools::Itertools;
use rand::{seq::SliceRandom, Rng};

use crate::{
    assets::{SpriteAssets, ATLAS_SIZE},
    data::{max_range, Persistent, SaveData},
    enemy::{get_enemy, Element},
    misc::{dir_to_vec, Direction},
    player::{Status, StatusEvent},
    GameState, PlayState, SCALE,
};

pub const TILE_SEP: f32 = 20.;
pub const ROOM_SEP: UVec2 = UVec2::new(15, 11);

// ······
// Plugin
// ······

pub struct TilemapPlugin;

impl Plugin for TilemapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Play), init).add_systems(
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
    tiles: HashSet<TileData>,
}

impl Tilemap {
    pub fn get_tile(&self, pos: IVec2) -> Option<Entity> {
        self.tiles.get(&TileData::pos(pos)).map(|t| t.entity)
    }
}

struct TileData {
    pub x: i32,
    pub y: i32,
    pub entity: Entity,
}

impl TileData {
    pub fn pos(p: IVec2) -> Self {
        Self {
            x: p.x,
            y: p.y,
            entity: Entity::PLACEHOLDER,
        }
    }
}

impl Hash for TileData {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.x.hash(state);
        self.y.hash(state);
    }
}

impl PartialEq for TileData {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

impl Eq for TileData {}

// ··········
// Components
// ··········

#[derive(Component, Default, PartialEq, Eq, Hash, Clone)]
pub enum Tile {
    #[default]
    Ground,
    Enemy,
    Path,
    Wall,
    LadderDown,
    LadderUp,
    Final,
}

// ·······
// Systems
// ·······

fn init(mut cmd: Commands, sprite_assets: Res<SpriteAssets>, save_data: Res<Persistent<SaveData>>) {
    let level = save_data.level;
    let tiles = generate_level(
        &mut cmd,
        &sprite_assets,
        level,
        (
            2 + (level * 0.3 as u32).clamp(0, 3),
            5 + (level * 0.5 as u32).clamp(0, 5),
        ),
        (ROOM_SEP.x / 2 + 1, ROOM_SEP.x - 4),
        (ROOM_SEP.y / 2 + 1, ROOM_SEP.y - 4),
    );
    cmd.insert_resource(Tilemap { tiles });
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
    if save_data.level >= max_range(save_data.range_level) {
        status_writer.send(StatusEvent(Status::ConnectionEmpty));
    } else if save_data.level + 2 >= max_range(save_data.range_level) {
        status_writer.send(StatusEvent(Status::ConnectionLow));
    }
}

// ·······
// Helpers
// ·······

pub fn pos_to_tile(pos: Vec2) -> IVec2 {
    let pos = pos / TILE_SEP / SCALE;
    IVec2::new(pos.x as i32, pos.y as i32)
}

pub fn tile_to_pos(pos: IVec2) -> Vec2 {
    Vec2::new(
        pos.x as f32 * TILE_SEP * SCALE,
        pos.y as f32 * TILE_SEP * SCALE,
    )
}

fn tile_to_index(tile: Tile) -> usize {
    let mut rng = rand::thread_rng();
    let tile = match tile {
        Tile::Ground => [0, 0, 1, 2, 5, 6, 7, ATLAS_SIZE.0 * 6 + 16]
            .choose(&mut rng)
            .unwrap(),
        Tile::Path => [1, 2, 3, 4].choose(&mut rng).unwrap(),
        Tile::Wall => [
            ATLAS_SIZE.0 * 13,
            ATLAS_SIZE.0 * 17 + 10,
            ATLAS_SIZE.0 * 17 + 13,
            ATLAS_SIZE.0 * 18 + 10,
        ]
        .choose(&mut rng)
        .unwrap(),
        Tile::LadderDown => &(ATLAS_SIZE.0 * 6 + 3),
        Tile::LadderUp => &(ATLAS_SIZE.0 * 6 + 2),
        _ => &0,
    };
    *tile
}

fn generate_level(
    cmd: &mut Commands,
    sprite_assets: &SpriteAssets,
    level: u32,
    rooms: (u32, u32),
    size_x: (u32, u32),
    size_y: (u32, u32),
) -> HashSet<TileData> {
    let mut rng = rand::thread_rng();

    let rooms = rng.gen_range(rooms.0..=rooms.1);
    let mut room_indices = HashSet::new();
    let mut room_pos = IVec2::ZERO;

    let mut tiles = HashMap::new();

    // Generate rooms
    for _ in 0..rooms {
        loop {
            if room_indices.insert(room_pos) {
                break;
            };
            let dir: Direction = rng.gen();
            let global_offset = dir_to_vec(&dir, 1.).as_ivec2();
            room_pos += global_offset;
        }

        let size = UVec2::new(
            rng.gen_range(size_x.0..=size_x.1),
            rng.gen_range(size_y.0..=size_y.1),
        );

        let offset = IVec2::new(
            rng.gen_range(0..(ROOM_SEP.x - size.x)) as i32 + room_pos.x * ROOM_SEP.x as i32,
            rng.gen_range(0..(ROOM_SEP.y - size.y)) as i32 + room_pos.y * ROOM_SEP.y as i32,
        );

        generate_room(&mut tiles, size, offset, level);
    }

    // Generate corridors
    let mut aux = room_indices.clone();
    for a in room_indices.clone() {
        for dir in Direction::iter() {
            let offset = dir_to_vec(dir, 1.).as_ivec2();
            let b = a + offset;
            let sep = match dir {
                Direction::North | Direction::South => ROOM_SEP.x * 2,
                Direction::East | Direction::West => ROOM_SEP.y * 2,
            };
            if aux.contains(&b) {
                // Corridor
                let center_a = a * ROOM_SEP.as_ivec2() + ROOM_SEP.as_ivec2() / 2;
                let mut first_wall = false;
                for pos in 0..sep {
                    let tile = TileData::pos(center_a + pos as i32 * offset);
                    // Find the first wall and start laying paths
                    if !first_wall {
                        if let Some(Tile::Wall) = tiles.get(&tile) {
                            first_wall = true;
                            tiles.insert(tile, Tile::Path);
                            continue;
                        }
                    }
                    // Lay paths until next wall
                    else if let Some(Tile::Wall) = tiles.insert(tile, Tile::Path) {
                        break;
                    }
                }
            }
        }
        aux.remove(&a);
    }

    // Insert ladder down or final key
    // This iterator is supposed to be random
    if level < 9 {
        for (_, tile) in tiles.iter_mut() {
            if !matches!(tile, Tile::Ground) {
                continue;
            }
            *tile = Tile::LadderDown;
            break;
        }
    }

    // Insert ladder up
    tiles.insert(
        TileData::pos(ROOM_SEP.as_ivec2() / 2),
        Tile::LadderUp,
    );

    // Create actual tiles
    let mut unique = false;
    tiles
        .iter()
        .map(|(k, v)| TileData {
            x: k.x,
            y: k.y,
            entity: create_tile(
                cmd,
                sprite_assets,
                level,
                &mut unique,
                IVec2::new(k.x, k.y),
                v.clone(),
                tile_to_index(v.clone()),
            ),
        })
        .collect()
}

fn generate_room(tiles: &mut HashMap<TileData, Tile>, size: UVec2, offset: IVec2, level: u32) {
    let mut rng = rand::thread_rng();
    let grow = (level / 3, level / 2);
    let num_enemies = rng.gen_range(1 + grow.0..3 + grow.1);

    let mut indices: Vec<(u32, u32)> = (1..=size.x).cartesian_product(0..=size.y).collect();
    indices.shuffle(&mut rand::thread_rng());
    let enemy_tiles = indices.get(0..num_enemies as usize);

    for (x, y) in (0..=size.x + 1).cartesian_product(0..=size.y + 1) {
        let tile = if x == 0 || x == size.x + 1 || y == 0 || y == size.y + 1 {
            Tile::Wall
        } else if let Some(tiles) = enemy_tiles {
            if tiles.contains(&(x, y)) {
                Tile::Enemy
            } else {
                Tile::Ground
            }
        } else {
            Tile::Ground
        };

        tiles.insert(
            TileData {
                x: x as i32 + offset.x,
                y: y as i32 + offset.y,
                entity: Entity::PLACEHOLDER,
            },
            tile,
        );
    }
}

fn create_tile(
    cmd: &mut Commands,
    sprite_assets: &SpriteAssets,
    level: u32,
    unique: &mut bool,
    pos: IVec2,
    tile: Tile,
    index: usize,
) -> Entity {
    // If it has an enemy, spawn it
    if matches!(tile, Tile::Enemy) {
        let (enemy, index) = get_enemy(pos, level, unique);
        cmd.spawn((
            SpriteBundle {
                transform: Transform::from_translation(tile_to_pos(pos).extend(5.))
                    .with_scale(Vec3::splat(SCALE)),
                texture: sprite_assets.one_bit.clone(),
                sprite: Sprite {
                    color: match enemy.elem {
                        Element::Basic => Color::srgb(0.812, 0.776, 0.722),
                        Element::Fire => Color::srgb(0.902, 0.282, 0.18),
                        Element::Water => Color::srgb(0.235, 0.675, 0.843),
                        Element::Grass => Color::srgb(0.22, 0.851, 0.451),
                    },
                    ..default()
                },
                ..default()
            },
            TextureAtlas {
                layout: sprite_assets.one_bit_atlas.clone(),
                index,
            },
            enemy,
            StateScoped(GameState::Play),
        ));
    }

    cmd.spawn((
        SpriteBundle {
            transform: Transform::from_translation(tile_to_pos(pos).extend(0.))
                .with_scale(Vec3::splat(SCALE)),
            texture: sprite_assets.one_bit.clone(),
            ..default()
        },
        TextureAtlas {
            layout: sprite_assets.one_bit_atlas.clone(),
            index,
        },
        tile,
        StateScoped(GameState::Play),
    ))
    .id()
}
