use std::f32::consts::PI;

use bevy::prelude::*;
#[cfg(feature = "persist")]
pub use bevy_persistent::prelude::Persistent;
use rand::Rng;

use crate::{
    assets::{CoreAssets, SoundAssets, ATLAS_SIZE},
    data::{attack, max_battery, SaveData},
    misc::{dir_to_vec, Direction, MoveTo, MIN_TURN_TIMER},
    player::Player,
    tilemap::{tile_to_pos, Tile, Tilemap},
    PlaySet, PlayState, TurnState,
};

const WEIGHTS: [[u32; 7]; 12] = [
    [80, 10, 00, 00, 00, 10, 0],
    [65, 20, 5, 00, 00, 10, 0],
    [35, 30, 25, 00, 00, 20, 0],
    [5, 40, 30, 5, 00, 20, 0],
    [00, 20, 45, 25, 00, 9, 1],
    [00, 10, 20, 50, 10, 9, 1],
    [00, 00, 5, 55, 20, 19, 0],
    [00, 00, 00, 60, 15, 19, 0],
    [00, 00, 00, 40, 35, 24, 0],
    [00, 00, 00, 20, 50, 29, 0],
    [00, 00, 00, 10, 70, 19, 0],
    [00, 00, 00, 5, 85, 9, 0],
];

// ······
// Plugin
// ······

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<DamageEvent>()
            .add_systems(OnEnter(TurnState::Enemy), enemy_turn)
            .add_systems(
                Update,
                update_enemies.run_if(in_state(TurnState::Enemy)),
            )
            .add_systems(
                Update,
                (
                    on_damage.in_set(PlaySet::Events),
                    (damage_text, enemy_flash).in_set(PlaySet::Animation),
                ),
            );
    }
}

// ··········
// Components
// ··········

pub enum EnemyType {
    Chicken,
    Cat,
    Dog,
    YoungOld, // for both kids and elders
    Man,
    Money,
    Battery,
    EndGame, // This is not an enemy, its a jewel
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq)]
pub enum Element {
    Basic,
    Fire,
    Water,
    Grass,
}

impl Element {
    pub fn next(&mut self) {
        *self = match self {
            Element::Basic => Element::Fire,
            Element::Fire => Element::Water,
            Element::Water => Element::Grass,
            Element::Grass => Element::Basic,
        };
    }

    pub fn prev(&mut self) {
        self.next();
        self.next();
        self.next(); // Esto é terrible
    }
}

#[derive(Component)]
pub struct Enemy {
    pub pos: IVec2,
    pub health: f32,
    pub typ: EnemyType,
    pub elem: Element,
}

#[derive(Component)]
struct EnemyTurn(Timer);

#[derive(Component)]
struct EnemyFlash(Timer);

#[derive(Component)]
struct DamageText(Timer, Vec2);

// ······
// Events
// ······

#[derive(Event)]
pub struct DamageEvent(pub Entity);

// ·······
// Systems
// ·······

fn on_damage(
    mut cmd: Commands,
    mut enemies: Query<&mut Enemy>,
    sound_assets: Res<SoundAssets>,
    mut damage_reader: EventReader<DamageEvent>,
    mut save_data: ResMut<Persistent<SaveData>>,
    mut next_play_state: ResMut<NextState<PlayState>>,
    assets: Res<CoreAssets>,
) {
    for DamageEvent(entity) in damage_reader.read() {
        cmd.entity(*entity)
            .try_insert(EnemyFlash(Timer::from_seconds(
                0.15,
                TimerMode::Once,
            )));

        if let Ok(mut enemy) = enemies.get_mut(*entity) {
            if let EnemyType::EndGame = enemy.typ {
                next_play_state.set(PlayState::GameWon);
                return;
            }

            if let EnemyType::Battery = enemy.typ {
                save_data.battery = (save_data.battery + max_battery(save_data.battery_level) / 4)
                    .clamp(0, max_battery(save_data.battery_level));
            }

            let value = match enemy.elem {
                Element::Basic => match save_data.attack_selected {
                    Element::Basic => attack(save_data.attack_level),
                    Element::Fire => {
                        if save_data.fire_uses > 0 {
                            save_data.fire_uses -= 1;
                            attack(save_data.attack_level)
                        } else {
                            0.
                        }
                    },
                    Element::Water => {
                        if save_data.water_uses > 0 {
                            save_data.water_uses -= 1;
                            attack(save_data.attack_level)
                        } else {
                            0.
                        }
                    },
                    Element::Grass => {
                        if save_data.grass_uses > 0 {
                            save_data.grass_uses -= 1;
                            attack(save_data.attack_level)
                        } else {
                            0.
                        }
                    },
                },
                Element::Fire => match save_data.attack_selected {
                    Element::Basic => attack(save_data.attack_level),
                    Element::Fire => {
                        if save_data.fire_uses > 0 {
                            save_data.fire_uses -= 1;
                            attack(save_data.attack_level)
                        } else {
                            0.
                        }
                    },
                    Element::Water => {
                        if save_data.water_uses > 0 {
                            save_data.water_uses -= 1;
                            attack(save_data.attack_level) * 1.5
                        } else {
                            0.
                        }
                    },
                    Element::Grass => {
                        if save_data.grass_uses > 0 {
                            save_data.grass_uses -= 1;
                            save_data.battery -= attack(save_data.attack_level) as u32;
                        }
                        0.
                    },
                },
                Element::Water => match save_data.attack_selected {
                    Element::Basic => attack(save_data.attack_level),
                    Element::Fire => {
                        if save_data.fire_uses > 0 {
                            save_data.fire_uses -= 1;
                            save_data.battery -= attack(save_data.attack_level) as u32;
                        }
                        0.
                    },
                    Element::Water => {
                        if save_data.water_uses > 0 {
                            save_data.water_uses -= 1;
                            attack(save_data.attack_level)
                        } else {
                            0.
                        }
                    },
                    Element::Grass => {
                        if save_data.grass_uses > 0 {
                            save_data.grass_uses -= 1;
                            attack(save_data.attack_level) * 1.5
                        } else {
                            0.
                        }
                    },
                },
                Element::Grass => match save_data.attack_selected {
                    Element::Basic => attack(save_data.attack_level),
                    Element::Fire => {
                        if save_data.fire_uses > 0 {
                            save_data.fire_uses -= 1;
                            attack(save_data.attack_level) * 1.5
                        } else {
                            0.
                        }
                    },
                    Element::Water => {
                        if save_data.water_uses > 0 {
                            save_data.water_uses -= 1;
                            save_data.battery -= attack(save_data.attack_level) as u32;
                        }
                        0.
                    },
                    Element::Grass => {
                        if save_data.grass_uses > 0 {
                            save_data.grass_uses -= 1;
                            attack(save_data.attack_level)
                        } else {
                            0.
                        }
                    },
                },
            };

            let value = value.clamp(0., enemy.health);
            enemy.health -= value;

            cmd.spawn((
                Text2dBundle {
                    text: Text::from_section(
                        if value > 0. { format!("{:.1}", value) } else { "X".into() },
                        TextStyle {
                            font: assets.font.clone(),
                            font_size: 10.,
                            color: enemy_color(&save_data.attack_selected).lighter(0.1),
                        },
                    ),
                    transform: Transform::from_translation(tile_to_pos(enemy.pos).extend(15.)),
                    ..default()
                },
                DamageText(
                    Timer::from_seconds(0.3, TimerMode::Once),
                    tile_to_pos(enemy.pos),
                ),
            ));

            if enemy.health <= 0. {
                cmd.entity(*entity).despawn();
                let mut rng = rand::thread_rng();
                cmd.spawn(AudioBundle {
                    source: match enemy.typ {
                        EnemyType::Chicken => sound_assets.chicken[rng.gen_range(0..2)].clone(),
                        EnemyType::Cat => sound_assets.cat[rng.gen_range(0..3)].clone(),
                        EnemyType::Dog => sound_assets.dog[rng.gen_range(0..3)].clone(),
                        EnemyType::YoungOld | EnemyType::Man => {
                            sound_assets.man[rng.gen_range(0..2)].clone()
                        },
                        EnemyType::EndGame | EnemyType::Money | EnemyType::Battery => {
                            sound_assets.upgrades[rng.gen_range(0..2)].clone()
                        },
                    },
                    settings: PlaybackSettings::DESPAWN,
                });
                save_data.money += match enemy.typ {
                    EnemyType::Chicken => rng.gen_range(4..6),
                    EnemyType::Cat => rng.gen_range(8..11),
                    EnemyType::Dog => rng.gen_range(14..17),
                    EnemyType::YoungOld => rng.gen_range(18..21),
                    EnemyType::Man => rng.gen_range(24..27),
                    EnemyType::Money => {
                        rng.gen_range((save_data.level + 2)..(save_data.level + 1) * 4)
                    },
                    EnemyType::EndGame | EnemyType::Battery => 0,
                };
                if !matches!(enemy.typ, EnemyType::Money) {
                    save_data.enemies_killed += 1;
                };
            }
        }

        cmd.spawn(AudioBundle {
            source: sound_assets.attack.clone(),
            settings: PlaybackSettings::DESPAWN,
        });
    }
}

fn enemy_turn(mut cmd: Commands) {
    cmd.spawn(EnemyTurn(Timer::from_seconds(
        MIN_TURN_TIMER,
        TimerMode::Once,
    )));
}

fn enemy_flash(
    mut cmd: Commands,
    mut enemies: Query<(
        Entity,
        &Enemy,
        &mut Sprite,
        &mut EnemyFlash,
    )>,
    time: Res<Time>,
) {
    for (entity, enemy, mut sprite, mut flash) in enemies.iter_mut() {
        let timer = flash.0.tick(time.delta());
        if timer.just_finished() {
            cmd.entity(entity).remove::<EnemyFlash>();
        }
        let n = (timer.fraction() * 5.) as u32;
        sprite.color = if n % 2 == 0 { Color::WHITE } else { enemy_color(&enemy.elem) };
    }
}

fn update_enemies(
    mut cmd: Commands,
    mut timer: Query<(Entity, &mut EnemyTurn)>,
    mut enemies: Query<(Entity, &mut Enemy, Option<&MoveTo>)>,
    player: Query<&Player>,
    mut tiles: Query<&mut Tile>,
    tilemap: Res<Tilemap>,
    time: Res<Time>,
    mut next_turn_state: ResMut<NextState<TurnState>>,
) {
    let Ok((entity, mut timer)) = timer.get_single_mut() else { return };
    let timer = timer.0.tick(time.delta());
    if timer.just_finished() {
        next_turn_state.set(TurnState::Player);
        cmd.entity(entity).despawn();
    }

    if timer.fraction() < 0.4 {
        return;
    };

    let Ok(player) = player.get_single() else {
        return;
    };

    let mut rng = rand::thread_rng();
    for (entity, mut enemy, move_to) in enemies.iter_mut() {
        match enemy.typ {
            EnemyType::EndGame | EnemyType::Money | EnemyType::Battery => continue,
            _ => {},
        };

        if move_to.is_some() {
            continue;
        };
        if rng.gen_bool(0.3) {
            cmd.entity(entity).insert(MoveTo::new(
                tile_to_pos(enemy.pos),
                tile_to_pos(enemy.pos),
                None,
            ));
            continue;
        };

        let dir: Direction = rng.gen();
        let pos = enemy.pos + dir_to_vec(&dir, 1.).as_ivec2();

        let Some(tile) = tilemap.get_tile(pos) else { continue };
        let Some(prev_tile) = tilemap.get_tile(enemy.pos) else { continue };
        let Ok(mut tile) = tiles.get_mut(tile) else { continue };
        let Tile::Ground = *tile else { continue };

        cmd.entity(entity).insert(MoveTo::new(
            tile_to_pos(enemy.pos),
            tile_to_pos(pos),
            if pos == player.pos { Some(dir) } else { None },
        ));

        if pos != player.pos {
            *tile = Tile::Enemy;

            enemy.pos = pos;

            let Ok(mut prev_tile) = tiles.get_mut(prev_tile) else { continue };
            *prev_tile = Tile::Ground;
        }
    }
}

fn damage_text(
    mut cmd: Commands,
    mut text: Query<(
        Entity,
        &mut Transform,
        &mut Text,
        &mut DamageText,
    )>,
    time: Res<Time>,
) {
    for (entity, mut trans, mut text, mut flash) in text.iter_mut() {
        let timer = flash.0.tick(time.delta());
        if timer.just_finished() {
            cmd.entity(entity).despawn();
        }
        let t = timer.fraction();
        text.sections[0].style.color.set_alpha(1. - t);
        let mut pos = flash.1;
        pos.x += (t * PI * 2.).sin() * 5.;
        pos.y += t * 50.;
        trans.translation = pos.extend(15.);
    }
}

// ·······
// Helpers
// ·······

pub fn get_enemy(pos: IVec2, level: u32, unique: &mut bool) -> (Enemy, usize) {
    let mut rng = rand::thread_rng();

    if level == 9 && !*unique {
        *unique = true;
        return (
            Enemy {
                pos,
                health: 1.,
                typ: EnemyType::EndGame,
                elem: Element::Basic,
            },
            45 + rng.gen_range(6..9) * ATLAS_SIZE.0,
        );
    }

    let typ = enemy_type(level);
    let (index, health) = match typ {
        EnemyType::Chicken => (
            7 * ATLAS_SIZE.0 + 25 + rng.gen_range(0..2),
            1.,
        ),
        EnemyType::Cat => (
            7 * ATLAS_SIZE.0 + 29 + rng.gen_range(0..2),
            2.,
        ),
        EnemyType::Dog => (7 * ATLAS_SIZE.0 + 31, 3.),
        EnemyType::YoungOld => (
            4 * ATLAS_SIZE.0 + 28 + rng.gen_range(0..2),
            4.,
        ),
        EnemyType::Man => (26 + rng.gen_range(0..6), 5.),
        EnemyType::Money => (10 * ATLAS_SIZE.0 + 33, 0.),
        EnemyType::Battery => (22 * ATLAS_SIZE.0 + 8, 0.),
        _ => unreachable!(),
    };

    (
        Enemy {
            pos,
            health,
            elem: match typ {
                EnemyType::Money | EnemyType::Battery => Element::Basic,
                _ => enemy_elem(),
            },
            typ,
        },
        index,
    )
}

fn enemy_type(level: u32) -> EnemyType {
    let rnd = rand::random::<u32>() % 100;
    let mut typ = 0;
    let mut cum_w = 0;
    for w in WEIGHTS[level as usize].iter() {
        cum_w += w;
        if rnd < cum_w {
            break;
        }
        typ += 1;
    }
    match typ {
        0 => EnemyType::Chicken,
        1 => EnemyType::Cat,
        2 => EnemyType::Dog,
        3 => EnemyType::YoungOld,
        4 => EnemyType::Man,
        5 => EnemyType::Money,
        _ => {
            if level > 5 && level <= 6 {
                EnemyType::Battery
            } else {
                EnemyType::Money
            }
        },
    }
}

fn enemy_elem() -> Element {
    let mut rng = rand::thread_rng();
    match rng.gen_range(0..4) {
        0 => Element::Fire,
        1 => Element::Water,
        2 => Element::Grass,
        _ => Element::Basic,
    }
}

pub fn enemy_color(elem: &Element) -> Color {
    match elem {
        Element::Basic => Color::srgb(0.812, 0.776, 0.722),
        Element::Fire => Color::srgb(0.902, 0.282, 0.18),
        Element::Water => Color::srgb(0.235, 0.675, 0.843),
        Element::Grass => Color::srgb(0.22, 0.851, 0.451),
    }
}
