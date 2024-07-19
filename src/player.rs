use bevy::prelude::*;

use crate::{
    assets::SpriteAssets,
    enemy::{DamageEvent, Enemy},
    input::{Action, ActionState},
    misc::{Direction, MoveTo},
    tilemap::{tile_to_pos, Tile, TileType, Tilemap},
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
            move_player.run_if(in_state(GameState::Play)),
        );
    }
}

// ··········
// Components
// ··········

#[derive(Component)]
pub struct Player {
    pub pos: UVec2,
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

fn move_player(
    mut cmd: Commands,
    input: Query<&ActionState<Action>>,
    mut player: Query<(Entity, &mut Player)>,
    enemies: Query<(Entity, &Enemy)>,
    tiles: Query<&Tile>,
    tilemap: Res<Tilemap>,
    mut damage_writer: EventWriter<DamageEvent>,
) {
    let Ok((entity, mut player)) = player.get_single_mut() else { return };
    let Ok(input) = input.get_single() else { return };

    // TODO: Change to uvec2
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

    let mut is_collision = false;
    for (enemy_entity, enemy) in enemies.iter() {
        if enemy.pos == UVec2::new(x, y) {
            is_collision = true;
            damage_writer.send(DamageEvent(enemy_entity));
            break;
        }
    }

    if !is_collision {
        let Some(tile) = tilemap.get_tile(x, y) else { return };
        let Ok(tile) = tiles.get(tile) else { return };
        is_collision = matches!(tile.tile, TileType::Collision);
    };

    cmd.entity(entity).insert(MoveTo::new(
        tile_to_pos(player.pos.x, player.pos.y),
        tile_to_pos(x, y),
        if is_collision { Some(dir) } else { None },
    ));

    if !is_collision {
        player.pos.x = x;
        player.pos.y = y;
    }
}
