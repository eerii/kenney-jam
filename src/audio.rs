//! Audio loading module

use bevy::{
    audio::{PlaybackMode, Volume},
    prelude::*,
};

use crate::{assets::SoundAssets, GameState};

// ······
// Plugin
// ······

/// Audio
/// Uses bevy audio to play music or sounds. This contains some examples on how
/// to set up audio, but it is disabled by default because audio varies greatly
/// from project to project.
pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Play), init.run_if(run_once()))
            .add_systems(Update, detect_audio_removal.run_if(in_state(GameState::Play)));
    }
}

// ··········
// Components
// ··········

#[derive(Component)]
struct AmbientMusic;

// ·······
// Systems
// ·······

/// Audio is added using component bundles
/// Using PlaybackSettings you can specify if it only plays once, if it loops or
/// even more complex behaviour, for example, to despawn the entity when the
/// audio is finished
/// CHANGE: Enable or disable background music and other sounds
fn init(mut cmd: Commands, assets: Res<SoundAssets>) {
    cmd.spawn((
        AudioBundle {
            source: assets.ambient_music.first().unwrap().clone(),
            settings: PlaybackSettings {
                mode: PlaybackMode::Despawn,
                volume: Volume::new(1.),
                ..default()
            },
        },
        AmbientMusic,
    ));
}

/// Detects when the audio entity is despawned and creates a new one
/// with a different part of the music loop
fn detect_audio_removal(
    mut cmd: Commands,
    assets: Res<SoundAssets>,
    mut removals: RemovedComponents<AmbientMusic>
    ) {
    for _ in removals.read() {
        let next = rand::random::<usize>() % (assets.ambient_music.len() - 1) + 1;
        cmd.spawn((
            AudioBundle {
                source: assets.ambient_music[next].clone(),
                settings: PlaybackSettings {
                    mode: PlaybackMode::Despawn,
                    volume: Volume::new(1.),
                    ..default()
                },
            },
            AmbientMusic,
        ));
    }
}
