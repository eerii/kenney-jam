use bevy::{
    color::palettes::css::{BLUE, GRAY, RED, SILVER, WHITE, YELLOW},
    prelude::*,
};
use rand::Rng;

use crate::{
    assets::SpriteAssets,
    data::{Persistent, SaveData},
    enemy::{DamageEvent, Enemy},
    input::{Action, ActionState},
    misc::{dir_to_vec, Direction, MoveTo},
    tilemap::{tile_to_pos, NextLevelEvent, Tile, TileType, Tilemap},
    GameState, PlaySet, SCALE,
};

// ······
// Plugin
// ······

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<StatusEvent>()
            .add_systems(OnEnter(GameState::Play), init)
            .add_systems(
                Update,
                (
                    tick_wrong_move.in_set(PlaySet::Tick),
                    move_player.in_set(PlaySet::Move),
                    check_player
                        .in_set(PlaySet::Collision)
                        .run_if(resource_changed::<Persistent<SaveData>>),
                    on_status.in_set(PlaySet::Events),
                ),
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

#[derive(Component)]
struct WrongMove(Timer);

// ······
// Events
// ······

pub enum Status {
    BatteryLow,
    BatteryEmpty,
    HealthLow,
    ConnectionLow,
    ConnectionEmpty,
}

#[derive(Event)]
pub struct StatusEvent(pub Status);

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
        StateScoped(GameState::Play), // Every time the level changes this entity is destroyed
    ));
}

fn move_player(
    mut cmd: Commands,
    mut player: Query<(Entity, &mut Player)>,
    enemies: Query<(Entity, &Enemy)>,
    input: Query<&ActionState<Action>>,
    tiles: Query<&Tile>,
    tilemap: Res<Tilemap>,
    mut save_data: ResMut<Persistent<SaveData>>,
    mut damage_writer: EventWriter<DamageEvent>,
    mut next_level_writer: EventWriter<NextLevelEvent>,
) {
    if save_data.battery == 0 {
        return;
    }

    let Ok((entity, mut player)) = player.get_single_mut() else {
        return;
    };
    let Ok(input) = input.get_single() else { return };

    let mut pos = player.pos;

    if !input.just_pressed(&Action::Move) {
        return;
    }
    let Some(axis) = input.clamped_axis_pair(&Action::Move) else { return };

    // Rooms left
    // 3 - 10%, 2 - 20%, 1 - 35%, 0 or more- 50%
    let rooms_left = save_data
        .max_range
        .saturating_sub(save_data.level)
        .clamp(0, 4);
    let percents = [0.5, 0.35, 0.2, 0.1, 0.0];
    let random_input = percents[rooms_left as usize] > rand::random::<f32>();

    let dir = if random_input {
        cmd.entity(entity).insert(WrongMove(Timer::from_seconds(
            0.1,
            TimerMode::Once,
        )));
        rand::thread_rng().gen()
    } else if axis.x().abs() > axis.y().abs() {
        if axis.x() > 0. {
            Direction::East
        } else {
            Direction::West
        }
    } else if axis.y() > 0. {
        Direction::North
    } else {
        Direction::South
    };

    let movement = dir_to_vec(&dir, 1.).as_ivec2();
    pos = (pos.as_ivec2() + movement)
        .clamp(IVec2::ZERO, IVec2::MAX)
        .as_uvec2();

    let mut is_collision = false;
    for (enemy_entity, enemy) in enemies.iter() {
        if enemy.pos == pos {
            is_collision = true;
            damage_writer.send(DamageEvent(enemy_entity));
            save_data.battery -= 1;
            break;
        }
    }

    if !is_collision {
        let Some(tile) = tilemap.get_tile(pos) else { return };
        let Ok(tile) = tiles.get(tile) else { return };
        if let TileType::Ladder = tile.tile {
            next_level_writer.send(NextLevelEvent);
            return;
        }
        is_collision = matches!(tile.tile, TileType::Collision);
    };

    cmd.entity(entity).insert(MoveTo::new(
        tile_to_pos(player.pos),
        tile_to_pos(pos),
        if is_collision { Some(dir) } else { None },
    ));

    if !is_collision {
        player.pos = pos;
        save_data.battery -= 1;
    }
}

fn check_player(save_data: Res<Persistent<SaveData>>, mut status_writer: EventWriter<StatusEvent>) {
    if save_data.battery < save_data.max_battery / 8 {
        status_writer.send(StatusEvent(Status::BatteryLow));
    }
    if save_data.battery == 0 {
        status_writer.send(StatusEvent(Status::BatteryEmpty));
    }
}

fn tick_wrong_move(
    mut cmd: Commands,
    time: Res<Time>,
    mut wrong_move: Query<(Entity, &mut Sprite, &mut WrongMove)>,
) {
    for (entity, mut sprite, mut component) in wrong_move.iter_mut() {
        let timer = component.0.tick(time.delta());
        if timer.just_finished() {
            sprite.color = WHITE.into();
            cmd.entity(entity).remove::<WrongMove>();
        }
    }
}

fn on_status(
    mut player: Query<(&mut Sprite, Option<&WrongMove>), With<Player>>,
    mut status_reader: EventReader<StatusEvent>,
) {
    let Ok((mut sprite, wrong_move)) = player.get_single_mut() else { return };

    for event in status_reader.read() {
        match event.0 {
            Status::BatteryLow => {
                sprite.color = YELLOW.into();
            },
            Status::BatteryEmpty => {
                sprite.color = RED.into();
            },
            Status::HealthLow => info!("health low"),
            Status::ConnectionLow => {
                sprite.color = SILVER.into();
            },
            Status::ConnectionEmpty => {
                sprite.color = GRAY.into();
            },
        };
    }

    if wrong_move.is_some() {
        sprite.color = BLUE.into();
    };
}
