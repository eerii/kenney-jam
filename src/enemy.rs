use bevy::prelude::*;

#[cfg(feature = "persist")]
pub use bevy_persistent::prelude::Persistent;

use crate::{
    assets::{SoundAssets, SpriteAssets, ATLAS_SIZE},
    tilemap::tile_to_pos,
    GameState, PlaySet, SCALE,
    data::SaveData,
};

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

const WEIGHTS: [[u32; 5]; 12] = [
    [ 90, 10, 00, 00, 00 ],
    [ 70, 25, 05, 00, 00 ],
    [ 40, 35, 25, 00, 00 ],
    [ 10, 50, 35, 05, 00 ],
    [ 00, 30, 45, 25, 00 ],
    [ 00, 10, 30, 50, 10 ],
    [ 00, 00, 10, 65, 25 ],
    [ 00, 00, 00, 65, 35 ],
    [ 00, 00, 00, 50, 50 ],
    [ 00, 00, 00, 30, 70 ],
    [ 00, 00, 00, 15, 85 ],
    [ 00, 00, 00, 05, 95 ],
];

pub enum EnemyType {
    Chicken,
    Cat,
    Dog,
    YoungOld,   // for both kids and elders
    Man,
}

fn ret_type(level: u32) -> EnemyType {
    let rnd = rand::random::<u32>() % 100;
    let mut typ = 0;
    let mut cum_w = 0;
    for w in WEIGHTS[level as usize].iter() {
        cum_w += w;
        if rnd < cum_w { break; }
        typ += 1;
    }
    match typ {
        0 => EnemyType::Chicken,
        1 => EnemyType::Cat,
        2 => EnemyType::Dog,
        3 => EnemyType::YoungOld,
        _ => EnemyType::Man
    }
}

#[derive(Component)]
pub struct Enemy {
    pub pos: UVec2,
    pub health: u32,
    pub typ: EnemyType,
}

// ······
// Events
// ······

#[derive(Event)]
pub struct DamageEvent(pub Entity);

// ·······
// Systems
// ·······

fn init(mut cmd: Commands,
        sprite_assets: Res<SpriteAssets>,
        save_data: Res<Persistent<SaveData>>,
) {
    for _ in 0..3 {
        let tile_pos = UVec2::new(
            rand::random::<u32>() % 11,
            rand::random::<u32>() % 7,
        );
        let pos = tile_to_pos(tile_pos);
        let index;
        let health;
        let typ = ret_type(save_data.level);
        match typ {
            EnemyType::Chicken => {
                index = 7 * ATLAS_SIZE.0 + 25 + rand::random::<usize>() % 2;
                health = 1;
            },
            EnemyType::Cat => {
                index = 7 * ATLAS_SIZE.0 + 29 + rand::random::<usize>() % 2;
                health = 2;
            },
            EnemyType::Dog => {
                index = 7 * ATLAS_SIZE.0 + 31;
                health = 3;
            },
            EnemyType::YoungOld => {
                index = 4 * ATLAS_SIZE.0 + 28 + rand::random::<usize>() % 2;
                health = 4;
            },
            EnemyType::Man => {
                index = 26 + rand::random::<usize>() % 6;
                health = 5;
            },
        }
        cmd.spawn((
            SpriteBundle {
                transform: Transform::from_translation(pos.extend(5.))
                    .with_scale(Vec3::splat(SCALE)),
                texture: sprite_assets.one_bit.clone(),
                ..default()
            },
            TextureAtlas {
                layout: sprite_assets.one_bit_atlas.clone(),
                index,
            },
            Enemy {
                pos: tile_pos,
                health,
                typ,
            },
            StateScoped(GameState::Play),
        ));
    }
}

fn on_damage(
    mut cmd: Commands,
    mut enemies: Query<&mut Enemy>,
    sound_assets: Res<SoundAssets>,
    mut damage_reader: EventReader<DamageEvent>,
) {
    for DamageEvent(entity) in damage_reader.read() {
        if let Ok(mut enemy) = enemies.get_mut(*entity) {
            enemy.health -= 1;
            if enemy.health == 0 {
                cmd.entity(*entity).despawn();
                cmd.spawn(AudioBundle {
                    // source: sound_assets.cat[rand::random::<usize>() % 3].clone(),
                    source: match enemy.typ {
                        EnemyType::Chicken => sound_assets.chicken[rand::random::<usize>() % 2].clone(),
                        EnemyType::Cat => sound_assets.cat[rand::random::<usize>() % 3].clone(),
                        EnemyType::Dog => sound_assets.dog[rand::random::<usize>() % 3].clone(),
                        EnemyType::YoungOld | EnemyType::Man => sound_assets.man[rand::random::<usize>() % 2].clone(),
                    },
                    settings: PlaybackSettings::DESPAWN,
                });
            }
        }

        cmd.spawn(AudioBundle {
            source: sound_assets.attack.clone(),
            settings: PlaybackSettings::DESPAWN,
        });
    }
}
