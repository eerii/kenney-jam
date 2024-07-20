use bevy::prelude::*;
#[cfg(feature = "persist")]
pub use bevy_persistent::prelude::Persistent;
use rand::Rng;

use crate::{
    assets::{SoundAssets, SpriteAssets, ATLAS_SIZE},
    data::SaveData,
    GameState, PlaySet,
};

const WEIGHTS: [[u32; 5]; 12] = [
    [90, 10, 00, 00, 00],
    [70, 25, 5, 00, 00],
    [40, 35, 25, 00, 00],
    [10, 50, 35, 5, 00],
    [00, 30, 45, 25, 00],
    [00, 10, 30, 50, 10],
    [00, 00, 10, 65, 25],
    [00, 00, 00, 65, 35],
    [00, 00, 00, 50, 50],
    [00, 00, 00, 30, 70],
    [00, 00, 00, 15, 85],
    [00, 00, 00, 5, 95],
];

// ······
// Plugin
// ······

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<DamageEvent>()
            .add_systems(OnEnter(GameState::Play), init)
            .add_systems(
                Update,
                on_damage.in_set(PlaySet::Events),
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
}

#[derive(serde::Deserialize, serde::Serialize)]
pub enum Element {
    Basic,
    Fire,
    Water,
    Grass,
}

#[derive(Component)]
pub struct Enemy {
    pub pos: IVec2,
    pub health: f32,
    pub typ: EnemyType,
    pub elem: Element,
}

// ······
// Events
// ······

#[derive(Event)]
pub struct DamageEvent(pub Entity);

// ·······
// Systems
// ·······

fn init(mut cmd: Commands, sprite_assets: Res<SpriteAssets>, save_data: Res<Persistent<SaveData>>) {
    // for _ in 0..3 {
    //     // TODO: Change generation
    //     let tile_pos = IVec2::new(
    //         rand::random::<i32>() % 11,
    //         rand::random::<i32>() % 7,
    //     );
    //     let pos = tile_to_pos(tile_pos);
    //
    //         // }
}

fn on_damage(
    mut cmd: Commands,
    mut enemies: Query<&mut Enemy>,
    sound_assets: Res<SoundAssets>,
    mut damage_reader: EventReader<DamageEvent>,
    mut save_data: ResMut<Persistent<SaveData>>,
) {
    for DamageEvent(entity) in damage_reader.read() {
        if let Ok(mut enemy) = enemies.get_mut(*entity) {
            enemy.health -= match enemy.elem {
                Element::Basic => match save_data.attack_selected {
                    Element::Basic => save_data.attack,
                    Element::Fire => {
                        if save_data.fire_uses > 0 {
                            save_data.fire_uses -= 1;
                            save_data.attack
                        } else { 0. }
                    },
                    Element::Water => {
                        if save_data.water_uses > 0 {
                            save_data.water_uses -= 1;
                            save_data.attack
                        } else { 0. }
                    },
                    Element::Grass => {
                        if save_data.grass_uses > 0 {
                            save_data.grass_uses -= 1;
                            save_data.attack
                        } else { 0. }
                    },
                },
                Element::Fire => match save_data.attack_selected {
                    Element::Basic => save_data.attack,
                    Element::Fire => {
                        if save_data.fire_uses > 0 {
                            save_data.fire_uses -= 1;
                            save_data.attack
                        } else { 0. }
                    },
                    Element::Water => {
                        if save_data.water_uses > 0 {
                            save_data.water_uses -= 1;
                            save_data.attack * 1.5
                        } else { 0. }
                    },
                    Element::Grass => {
                        if save_data.grass_uses > 0 {
                            save_data.grass_uses -= 1;
                        }
                        0.
                    },
                },
                Element::Water => match save_data.attack_selected {
                    Element::Basic => save_data.attack,
                    Element::Fire => {
                        if save_data.fire_uses > 0 {
                            save_data.fire_uses -= 1;
                        }
                        0.
                    },
                    Element::Water => {
                        if save_data.water_uses > 0 {
                            save_data.water_uses -= 1;
                            save_data.attack
                        } else { 0. }
                    },
                    Element::Grass => {
                        if save_data.grass_uses > 0 {
                            save_data.grass_uses -= 1;
                            save_data.attack * 1.5
                        } else { 0. }
                    },
                },
                Element::Grass => match save_data.attack_selected {
                    Element::Basic => save_data.attack,
                    Element::Fire => {
                        if save_data.fire_uses > 0 {
                            save_data.fire_uses -= 1;
                            save_data.attack * 1.5
                        } else { 0. }
                    },
                    Element::Water => {
                        if save_data.water_uses > 0 {
                            save_data.water_uses -= 1;
                        }
                        0.
                    },
                    Element::Grass => {
                        if save_data.grass_uses > 0 {
                            save_data.grass_uses -= 1;
                            save_data.attack
                        } else { 0. }
                    },
                },
            };
            if enemy.health <= 0. {
                cmd.entity(*entity).despawn();
                let mut rng = rand::thread_rng();
                cmd.spawn(AudioBundle {
                    source: match enemy.typ {
                        EnemyType::Chicken => {
                            sound_assets.chicken[rng.gen_range(0..2)].clone()
                        },
                        EnemyType::Cat => sound_assets.cat[rng.gen_range(0..3)].clone(),
                        EnemyType::Dog => sound_assets.dog[rng.gen_range(0..3)].clone(),
                        EnemyType::YoungOld | EnemyType::Man => {
                            sound_assets.man[rng.gen_range(0..2)].clone()
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
                };
            }
        }

        cmd.spawn(AudioBundle {
            source: sound_assets.attack.clone(),
            settings: PlaybackSettings::DESPAWN,
        });
    }
}

// ·······
// Helpers
// ·······

pub fn get_enemy(pos: IVec2, level: u32) -> (Enemy, usize) {
    let mut rng = rand::thread_rng();
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
    };

    (Enemy { pos, health, typ, elem: enemy_elem() }, index)
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
        _ => EnemyType::Man,
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
