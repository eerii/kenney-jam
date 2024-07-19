use std::f32::consts::PI;

use bevy::prelude::*;

use crate::{
    assets::SpriteAssets,
    input::{Action, ActionState},
    tilemap::{tile_to_pos, Tile, TileType, Tilemap, TILE_SEP},
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
            (update_player, move_to)
                .chain()
                .run_if(in_state(GameState::Play)),
        );
    }
}

// ··········
// Components
// ··········

#[derive(Component)]
struct Player {
    pos: UVec2,
}

enum Direction {
    North,
    South,
    East,
    West,
}

#[derive(Component)]
struct MoveTo {
    start: Vec2,
    target: Vec2,
    bump_dir: Option<Direction>,
    timer: Timer,
}

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
        Player {
            pos: UVec2::new(5, 3),
        },
    ));
}

fn update_player(
    mut cmd: Commands,
    input: Query<&ActionState<Action>>,
    mut player: Query<(Entity, &mut Player)>,
    tiles: Query<&Tile>,
    tilemap: Res<Tilemap>,
) {
    let Ok((entity, mut player)) = player.get_single_mut() else { return };
    let Ok(input) = input.get_single() else { return };

    // let (mut x, mut y) = pos_to_tile(trans.translation.truncate());
    let (mut x, mut y) = (player.pos.x, player.pos.y);

    if !input.just_pressed(&Action::Move) {
        return;
    }

    let Some(axis) = input.clamped_axis_pair(&Action::Move) else { return };
    let dir = if axis.x().abs() > axis.y().abs() {
        if axis.x() > 0. {
            x = x.saturating_add(1);
            Direction::East
        } else {
            x = x.saturating_sub(1);
            Direction::West
        }
    } else if axis.y() > 0. {
        y = y.saturating_add(1);
        Direction::North
    } else {
        y = y.saturating_sub(1);
        Direction::South
    };

    let Some(tile) = tilemap.get_tile(x, y) else { return };
    let Ok(tile) = tiles.get(tile) else { return };
    let is_collision = matches!(tile.tile, TileType::Collision);

    cmd.entity(entity).insert(MoveTo {
        start: tile_to_pos(player.pos.x, player.pos.y),
        target: tile_to_pos(x, y),
        bump_dir: if is_collision { Some(dir) } else { None },
        timer: Timer::from_seconds(0.15, TimerMode::Once),
    });

    if !is_collision {
        player.pos.x = x;
        player.pos.y = y;
    }
}

fn move_to(
    mut cmd: Commands,
    time: Res<Time>,
    mut movables: Query<(Entity, &mut MoveTo, &mut Transform)>,
) {
    for (entity, mut to, mut trans) in movables.iter_mut() {
        let timer = to.timer.tick(time.delta());
        if timer.just_finished() {
            cmd.entity(entity).remove::<MoveTo>();
        }
        let t = timer.fraction();

        let pos = if let Some(dir) = &to.bump_dir {
            let offset = (t * PI).sin() * TILE_SEP;
            to.start + dir_to_vec(dir, offset)
        } else {
            to.start.lerp(to.target, t)
        };
        trans.translation = pos.extend(trans.translation.z);
    }
}

// ·······
// Helpers
// ·······

fn dir_to_vec(dir: &Direction, val: f32) -> Vec2 {
    match dir {
        Direction::North => Vec2::new(0., val),
        Direction::South => Vec2::new(0., -val),
        Direction::East => Vec2::new(val, 0.),
        Direction::West => Vec2::new(-val, 0.),
    }
}
