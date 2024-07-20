//! Audio loading module

use bevy::{
    audio::{PlaybackMode, Volume},
    prelude::*,
};

use crate::{assets::SoundAssets, PlayState};

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
        app.add_systems(OnEnter(PlayState::Menu), init_menu)
            .add_systems(OnExit(PlayState::Menu), exit_menu)
            .add_systems(
                OnExit(PlayState::Play),
                exit_play.run_if(in_state(crate::GameState::Play)),
            );

        app.add_systems(OnEnter(PlayState::Play), init_play)
            .add_systems(
                Update,
                (
                    detect_audio_removal.run_if(in_state(PlayState::Play)),
                    fade_out,
                ),
            );
    }
}

// ··········
// Components
// ··········

#[derive(Component)]
struct AmbientMusic;

#[cfg(feature = "menu")]
#[derive(Component)]
struct MainMusic;

#[derive(Component)]
struct FadeOut {
    despawn: bool,
}

// ·······
// Systems
// ·······

/// Audio is added using component bundles
/// Using PlaybackSettings you can specify if it only plays once, if it loops or
/// even more complex behaviour, for example, to despawn the entity when the
/// audio is finished
/// CHANGE: Enable or disable background music and other sounds
fn init_play(
    mut cmd: Commands,
    ambient: Query<(Entity, &AudioSink), With<AmbientMusic>>,
    assets: Res<SoundAssets>,
) {
    match ambient.get_single() {
        Ok((entity, ambient)) => {
            ambient.play();
            ambient.set_volume(1.);
            cmd.entity(entity).remove::<FadeOut>();
        },
        Err(_) => {
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
        },
    };
}

#[cfg(feature = "menu")]
fn exit_play(
    mut cmd: Commands,
    state: Res<State<PlayState>>,
    sink: Query<Entity, With<AmbientMusic>>,
) {
    if matches!(state.get(), PlayState::Menu) {
        for entity in sink.iter() {
            cmd.entity(entity).insert(FadeOut { despawn: false });
        }
    }
}

/// Detects when the audio entity is despawned and creates a new one
/// with a different part of the music loop
fn detect_audio_removal(
    mut cmd: Commands,
    assets: Res<SoundAssets>,
    mut removals: RemovedComponents<AmbientMusic>,
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
fn init_menu(mut cmd: Commands, assets: Res<SoundAssets>) {
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

#[cfg(feature = "menu")]
fn exit_menu(mut cmd: Commands, sink: Query<Entity, With<MainMusic>>) {
    for entity in sink.iter() {
        cmd.entity(entity).insert(FadeOut { despawn: true });
    }
}

fn fade_out(mut cmd: Commands, sink: Query<(Entity, &AudioSink, &FadeOut)>, time: Res<Time>) {
    for (entity, audio, fade_out) in sink.iter() {
        audio.set_volume(audio.volume() - time.delta_seconds() / 1.5);
        if audio.volume() <= 0. {
            if fade_out.despawn {
                cmd.entity(entity).despawn_recursive();
            } else {
                audio.pause();
                cmd.entity(entity).remove::<FadeOut>();
            }
        }
    }
}
