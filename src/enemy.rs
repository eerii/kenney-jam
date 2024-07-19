use bevy::prelude::*;

use crate::{
    assets::{SoundAssets, SpriteAssets},
    tilemap::tile_to_pos,
    GameState, PlaySet, SCALE,
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

#[derive(Component)]
pub struct Enemy {
    pub pos: UVec2,
    pub health: u32,
}

// ······
// Events
// ······

#[derive(Event)]
pub struct DamageEvent(pub Entity);

// ·······
// Systems
// ·······

fn init(mut cmd: Commands, sprite_assets: Res<SpriteAssets>) {
    for _ in 0..3 {
        let (x, y) = (
            rand::random::<u32>() % 11,
            rand::random::<u32>() % 7,
        );
        let pos = tile_to_pos(x, y);
        cmd.spawn((
            SpriteBundle {
                transform: Transform::from_translation(pos.extend(5.))
                    .with_scale(Vec3::splat(SCALE)),
                texture: sprite_assets.one_bit.clone(),
                ..default()
            },
            TextureAtlas {
                layout: sprite_assets.one_bit_atlas.clone(),
                index: 29 + 7 * 48,
            },
            Enemy {
                pos: UVec2::new(x, y),
                health: 2,
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
            }
        }

        cmd.spawn(AudioBundle {
            source: sound_assets.boing.clone(),
            settings: PlaybackSettings::DESPAWN,
        });
    }
}
