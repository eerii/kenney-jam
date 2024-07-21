//! Data persistence module

use bevy::prelude::*;
#[cfg(feature = "persist")]
pub use bevy_persistent::prelude::Persistent;
use serde::{Deserialize, Serialize};

#[cfg(not(feature = "persist"))]
pub use self::alt::Persistent;
use crate::{enemy::Element, GameState, PlayState};

// ······
// Plugin
// ······

/// Data persistence
/// Used to create persistent serialized files with options or save data
/// It saves and loads from toml any resource that needs to survive app reloads
pub struct DataPlugin;

impl Plugin for DataPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<RestartEvent>()
            .add_systems(OnEnter(GameState::Startup), init_data)
            .add_systems(
                OnEnter(GameState::Play),
                restart.run_if(run_once()),
            )
            .add_systems(Update, on_restart);
    }
}

// ·········
// Resources
// ·········

/// Game options
/// Useful for accesibility and the settings menu
/// CHANGE: Add any configurable game options here
#[derive(Resource, Serialize, Deserialize)]
pub struct GameOptions {
    /// Base color of the game, used for backgrounds, etc
    pub base_color: Color,
    /// Accent color, meant to contrast with the base color
    pub accent_color: Color,

    /// Controlls if text to speech is enabled for menu navigation
    #[cfg(feature = "tts")]
    pub text_to_speech: bool,
}

impl Default for GameOptions {
    fn default() -> Self {
        Self {
            base_color: Color::srgb(0.3, 0.5, 0.9),
            accent_color: Color::srgb(0.3, 0.5, 0.9),
            #[cfg(feature = "tts")]
            text_to_speech: default(),
        }
    }
}

/// Save data
/// A place to save the player's progress
/// CHANGE: Add relevant save data here
#[derive(Resource, Serialize, Deserialize)]
pub struct SaveData {
    pub level: u32,
    pub battery: u32,
    pub range_level: usize,
    pub battery_level: usize,
    pub attack_level: usize,
    pub fire: u32, // don't need level, use this as level
    pub water: u32,
    pub grass: u32,
    pub fire_uses: u32,
    pub water_uses: u32,
    pub grass_uses: u32,
    pub attack_selected: Element,
    pub money: u32,
    pub enemies_killed: u32,
    pub levels_completed: u32,
    pub deaths: u32,
}

impl Default for SaveData {
    fn default() -> Self {
        Self {
            level: 0,
            battery: 200,
            range_level: 1,
            battery_level: 1,
            attack_level: 1,
            fire: 0,
            water: 0,
            grass: 0,
            fire_uses: 0,
            water_uses: 0,
            grass_uses: 0,
            attack_selected: Element::Basic,
            money: 0,
            enemies_killed: 0,
            levels_completed: 0,
            deaths: 0,
        }
    }
}

#[inline]
pub fn max_range(level: usize) -> u32 {
    (4 + level) as u32
}

#[inline]
pub fn max_battery(level: usize) -> u32 {
    (25 + level * 50) as u32
}

#[inline]
pub fn attack(level: usize) -> f32 {
    0.5 + level as f32 * 0.5
}

/// When persist is not enabled, this wrapper just serves
/// as a placeholder to allow to use the resouces regularlly
#[cfg(not(feature = "persist"))]
mod alt {
    use std::ops::{Deref, DerefMut};

    use super::*;

    /// Placeholder persistent resource for when the persist feature is disabled
    /// This does nothing, just derefs to the inner value
    #[derive(Resource)]
    pub struct Persistent<T>(pub T);

    impl<T> Deref for Persistent<T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl<T> DerefMut for Persistent<T> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }

    impl<T> Persistent<T> {
        /// Updates the inner resource with a closure
        #[allow(clippy::result_unit_err)]
        pub fn update(&mut self, updater: impl Fn(&mut T)) -> Result<(), ()> {
            updater(&mut self.0);
            Ok(())
        }
    }
}

// ······
// Events
// ······

#[derive(Event)]
pub struct RestartEvent;

// ·······
// Systems
// ·······

#[cfg(feature = "persist")]
pub(crate) fn init_data(mut cmd: Commands) {
    let path = std::path::Path::new(if cfg!(target_arch = "wasm32") { "local" } else { ".data" });
    info!("{:?}", path);

    cmd.insert_resource(
        Persistent::<GameOptions>::builder()
            .name("game options")
            .format(bevy_persistent::StorageFormat::Toml)
            .path(path.join("options.toml"))
            .default(GameOptions::default())
            .revertible(true)
            .revert_to_default_on_deserialization_errors(true)
            .build()
            .expect("failed to initialize game options"),
    );

    cmd.insert_resource(
        Persistent::<SaveData>::builder()
            .name("save data")
            .format(bevy_persistent::StorageFormat::Toml)
            .path(path.join("save.toml"))
            .default(SaveData::default())
            .revertible(true)
            .revert_to_default_on_deserialization_errors(true)
            .build()
            .expect("failed to initialize save data"),
    );
}

pub(crate) fn restart(mut reset_writer: EventWriter<RestartEvent>) {
    reset_writer.send(RestartEvent);
}

#[cfg(not(feature = "persist"))]
pub(crate) fn init_data(mut cmd: Commands) {
    cmd.insert_resource(Persistent(GameOptions::default()));
    cmd.insert_resource(Persistent(SaveData::default()));
}

fn on_restart(
    mut next_state: ResMut<NextState<GameState>>,
    mut next_play_state: ResMut<NextState<PlayState>>,
    mut save_data: ResMut<Persistent<SaveData>>,
    mut restart_reader: EventReader<RestartEvent>,
    mut first: Local<bool>,
) {
    if restart_reader.read().next().is_some() {
        next_state.set(GameState::Play);
        if !*first {
            *first = true
        } else {
            next_play_state.set(PlayState::Play);
        }
        let battery = max_battery(save_data.battery_level);
        let _ = save_data.update(|data| {
            data.level = 0;
            data.battery = battery;
            data.fire_uses = data.fire;
            data.water_uses = data.water;
            data.grass_uses = data.grass;
        });
    }
}
