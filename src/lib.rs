//! An opinionated template for bevy games

// CHANGE: Comment this if it's too anoying when making games
// #![warn(missing_docs)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]

pub mod assets;
pub mod audio;
pub mod camera;
pub mod data;
pub mod enemy;
#[cfg(feature = "input")]
pub mod input;
pub mod misc;
pub mod player;
pub mod tilemap;
#[cfg(feature = "ui")]
pub mod ui;

use bevy::{log::LogPlugin, prelude::*, window::WindowResolution};

const GAME_RES: Vec2 = Vec2::new(256., 192.);
const SCALE: f32 = 4.;

// const GAME_RES: Vec2 = Vec2::new(256. * 4., 192. * 4.);
// const SCALE: f32 = 1.;

/// Indicates at which point the game is. Very useful for controlling which
/// systems run when (in_state) and to create transitions (OnEnter/OnExit)
/// You can also scope entities to a state with StateScoped, and they will
/// be deleted automatically when the state ends
#[derive(States, Debug, Default, Clone, Eq, PartialEq, Hash)]
pub enum GameState {
    /// The game starts on the setup state
    /// This runs before *anything*, including Startup
    /// It inmediately transitions to loading, so only use it for OnEnter
    #[default]
    Startup,
    /// After startup it transitions to loading, which handles splash screens
    /// and assets. It stays here until all the relevant assets are ready
    Loading,
    /// Main state, this represents the actual game
    Play,
    /// Going to another level
    LevelTransition,
    /// Upgrade shop
    Shop,
    UpdateShop,
    /// End of the `Play` state, useful to restart the game
    End,
}

#[derive(SubStates, Default, Debug, Clone, Eq, PartialEq, Hash)]
#[source(GameState = GameState::Play)]
pub enum PlayState {
    Play,
    #[default]
    Menu,
    ToShop,
    ToLevel,
    GameWon,
    GameOver,
}

#[derive(SubStates, Debug, Default, Clone, Eq, PartialEq, Hash)]
#[source(PlayState = PlayState::Play)]
pub enum TurnState {
    #[default]
    Player,
    Enemy,
}

/// Custom update schedules
/// Useful for ordering inside Update
#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash)]
enum PlaySet {
    Tick,
    Move,
    Collision,
    Events,
    Animation,
}

/// Static configuration
/// Allows to pass options to the game plugin such as the title and resolution.
/// Must be added before the plugin
/// CHANGE: You can customize the default parameters of the game here
#[derive(Resource, Clone)]
pub struct AppConfig {
    /// The title on the main window
    pub game_title: &'static str,
    /// What size should the main window open in
    pub initial_window_res: WindowResolution,
    /// The size of the canvas that renders a pixel perfect game
    #[cfg(feature = "pixel_perfect")]
    pub initial_game_res: Vec2,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            game_title: "Nightmare in Silver",
            initial_window_res: (GAME_RES * SCALE).into(),
            #[cfg(feature = "pixel_perfect")]
            initial_game_res: GAME_RES,
        }
    }
}

/// Main game plugin
/// This template is structured using plugins. A plugin makes changes to the
/// app, usually adding systems and resources. This is the main plugin that
/// initializes all subsistems. Each plugin is defined in a submodule (mod ***)
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        // Get previous app configuration or create a new one
        let config: &AppConfig;
        if let Some(res) = app.world().get_resource::<AppConfig>() {
            config = res;
        } else {
            app.insert_resource(AppConfig::default());
            config = app.world().resource::<AppConfig>();
        }

        // Window
        // Controls initial resolution, resizing
        let window_plugin = WindowPlugin {
            primary_window: Some(Window {
                title: config.game_title.into(),
                resolution: config.initial_window_res.clone(),
                resizable: cfg!(feature = "resizable"),
                canvas: Some("#bevy".to_string()),
                prevent_default_event_handling: false,
                ..default()
            }),
            ..default()
        };

        // Image
        // Sets the interpolation (nearest for pixel art, default otherwise)
        let image_plugin = if cfg!(feature = "pixel_perfect") {
            ImagePlugin::default_nearest()
        } else {
            ImagePlugin::default()
        };

        // Log
        // Modifies the logging to the console. More verbose when running debug builds
        let default_log = "info,wgpu_core=error,wgpu_hal=error,bevy_alt_ui_navigation_lite=error";
        let log_plugin = if cfg!(debug_assertions) {
            LogPlugin {
                level: bevy::log::Level::DEBUG,
                filter: format!("{},kenney_jam=debug", default_log),
                ..default()
            }
        } else {
            LogPlugin {
                level: bevy::log::Level::INFO,
                filter: default_log.into(),
                ..default()
            }
        };

        // Asset
        // In the future, it will use processed assets with Bevy Asset v2.
        // For now this is disabled since it is very early in development
        let asset_plugin = AssetPlugin {
            // mode: AssetMode::Processed,
            ..default()
        };

        // Use asset embedder if enabled
        #[cfg(feature = "bevy_embedded_assets")]
        {
            use bevy_embedded_assets::{EmbeddedAssetPlugin, PluginMode};
            app.add_plugins(EmbeddedAssetPlugin {
                mode: PluginMode::ReplaceDefault,
            });
        }

        // Add default bevy plugins with our overrides
        app.add_plugins(
            DefaultPlugins
                .set(window_plugin)
                .set(image_plugin)
                .set(log_plugin)
                .set(asset_plugin),
        );

        // Add the detailed schedule set
        app.configure_sets(
            Update,
            (
                PlaySet::Tick,
                PlaySet::Move,
                PlaySet::Collision,
                PlaySet::Events,
                PlaySet::Animation,
            )
                .chain()
                .run_if(in_state(PlayState::Play)),
        );

        // Insert the game state
        app.insert_state(GameState::default())
            .enable_state_scoped_entities::<GameState>()
            .add_sub_state::<PlayState>()
            .enable_state_scoped_entities::<PlayState>()
            .add_sub_state::<TurnState>();

        // Add the rest of the plugins
        app.add_plugins((
            assets::AssetLoaderPlugin,
            audio::AudioPlugin,
            camera::CameraPlugin,
            data::DataPlugin,
            enemy::EnemyPlugin,
            misc::MiscPlugin,
            player::PlayerPlugin,
            tilemap::TilemapPlugin,
        ));

        #[cfg(feature = "input")]
        app.add_plugins(input::InputPlugin);

        #[cfg(feature = "ui")]
        app.add_plugins(ui::UiPlugin);

        app.add_systems(
            Update,
            finish_setup.run_if(in_state(GameState::Startup)),
        );
    }
}

// ·······
// Systems
// ·······

/// This system inmediately transitions Startup to Loading, ensuring that the
/// first only lasts for a frame and that only the OnEnter and OnExit schedules
/// are valid. This is a replacement for PreStartup and PostStartup that works
/// with the new 0.14 schedule ordering.
fn finish_setup(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::Loading);
}
