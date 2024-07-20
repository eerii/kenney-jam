//! Audio loading module

use bevy::{
    audio::{PlaybackMode, Volume},
    prelude::*,
};

use crate::{assets::SoundAssets, GameState, PlayState};

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
        #[cfg(feature = "menu")]
        app.add_systems(OnEnter(PlayState::Menu), menu_music)
            .add_systems(OnExit(PlayState::Menu), fade_out);
        app.add_systems(OnEnter(PlayState::Play), init.run_if(run_once()))
            .add_systems(Update, detect_audio_removal.run_if(in_state(PlayState::Play)));
    }
}

// ··········
// Components
// ··········

#[derive(Component)]
struct AmbientMusic;

#[derive(Component)]
struct MainMusic;

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

#[cfg(feature = "menu")]
fn menu_music(mut cmd: Commands, assets: Res<SoundAssets>) {
    cmd.spawn((
        AudioBundle {
            source: assets.main_menu.clone(),
            settings: PlaybackSettings {
                mode: PlaybackMode::Loop,
                volume: Volume::new(1.),
                ..default()
            },
        },
        MainMusic,
    ));
}

fn fade_out(
    mut cmd: Commands,
    mut sink: Query<(&mut AudioSink, Entity), With<MainMusic>>,
    time: Res<Time>,
) {
    for (audio, entity) in sink.iter_mut() {
        audio.set_volume(audio.volume() - time.delta_seconds()/0.5);
        if audio.volume() <= 0. {
            cmd.entity(entity).despawn_recursive();
        }
    }
}
