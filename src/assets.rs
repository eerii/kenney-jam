//! Asset loading module

use bevy::prelude::*;

use crate::GameState;

pub const ATLAS_SIZE: (usize, usize) = (49, 23);

// ······
// Plugin
// ······

/// Asset loader
/// Creates asset collections and keeps track of their loading state
/// Once they are done, it exits GameState::Loading
pub struct AssetLoaderPlugin;

impl Plugin for AssetLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(LoadingData::default())
            .add_systems(OnEnter(GameState::Startup), load_core)
            .add_systems(
                OnEnter(GameState::Loading),
                (load_sound, load_sprites),
            )
            .add_systems(
                Update,
                check_load_state.run_if(in_state(GameState::Loading)),
            );
    }
}

// ·········
// Resources
// ·········

/// Assets for the splash screen and menus
/// They are loaded inmediately after the app is fired, so they have no effect
/// on loading state
#[derive(Resource)]
pub struct CoreAssets {
    /// Icon of the bevy engine, used in splash screens and examples
    pub bevy_icon: Handle<Image>,
    /// Default font for the text in the engine
    pub font: Handle<Font>,
}

/// Sprite assets
/// They are loaded during the loading state, showing the progress
#[derive(Resource)]
pub struct SpriteAssets {
    pub one_bit: Handle<Image>,
    pub one_bit_atlas: Handle<TextureAtlasLayout>,
}

/// Sound assets
/// They are loaded during the loading state, showing the progress
#[derive(Resource)]
pub struct SoundAssets {
    /// Simple jumping sound
    pub boing: Handle<AudioSource>,
    /// Main menu music
    pub main_menu: Handle<AudioSource>,
    /// Background music
    pub ambient_music: Vec<Handle<AudioSource>>,
}

// ·······
// Systems
// ·······

pub(crate) fn load_core(mut cmd: Commands, asset_server: Res<AssetServer>) {
    // They use the asset server directly
    let assets = CoreAssets {
        bevy_icon: asset_server.load("icons/bevy.png"),
        font: asset_server.load("fonts/sans.ttf"),
    };

    cmd.insert_resource(assets);
}

fn load_sprites(
    mut cmd: Commands,
    asset_server: Res<AssetServer>,
    mut loading_data: ResMut<LoadingData>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let layout = TextureAtlasLayout::from_grid(
        UVec2::splat(16),
        ATLAS_SIZE.0 as u32,
        ATLAS_SIZE.1 as u32,
        None,
        None,
    );

    let assets = SpriteAssets {
        one_bit: loading_data.load(&asset_server, "sprites/1bit.png"),
        one_bit_atlas: texture_atlas_layouts.add(layout),
    };

    cmd.insert_resource(assets);
}

fn load_sound(
    mut cmd: Commands,
    asset_server: Res<AssetServer>,
    mut loading_data: ResMut<LoadingData>,
) {
    let music = [
        "music/intro.ogg",
        "music/A_no_piano.ogg",
        "music/A_piano.ogg",
        "music/B_no_piano.ogg",
        "music/B_piano.ogg",
    ];
    // They use the loading data manager, which tracks if they are loaded
    let assets = SoundAssets {
        boing: loading_data.load(&asset_server, "sounds/boing.ogg"),
        main_menu: loading_data.load(&asset_server, "music/main_menu.ogg"),
        ambient_music: loading_data.load_vec(&asset_server, &music),
    };

    cmd.insert_resource(assets);
}

// ·······
// Helpers
// ·······

#[derive(Resource, Debug, Default)]
pub(crate) struct LoadingData {
    assets: Vec<UntypedHandle>,
    loaded: usize,
    total: usize,
}

impl LoadingData {
    /// Loads an asset into the server and adds it to the list to keep track of
    /// its state
    fn load<T: Asset>(&mut self, asset_server: &AssetServer, path: &'static str) -> Handle<T> {
        let handle = asset_server.load(path);

        self.assets.push(handle.clone().into());
        self.total += 1;

        handle
    }

    // same as previous load but can be passed an array of paths
    fn load_vec<T: Asset>(&mut self, asset_server: &AssetServer, paths: &[&'static str]) -> Vec<Handle<T>> {
        let mut handles = Vec::new();

        for path in paths {
            println!("Loading {}", path);
            handles.push(self.load(asset_server, path));
        }

        handles
    }

    /// Returns the current loaded assets and the total assets registered
    pub(crate) fn current(&mut self, asset_server: &AssetServer) -> (usize, usize) {
        // Find assets that have already been loaded and remove them from the list
        self.assets.retain(|asset| {
            let Some(state) = asset_server.get_load_states(asset) else { return true };

            let bevy::asset::RecursiveDependencyLoadState::Loaded = state.2 else {
                return true;
            };

            self.loaded += 1;
            debug!(
                "\"{:?}\" loaded! ({}/{})",
                asset.path(),
                self.loaded,
                self.total
            );
            false
        });

        (self.loaded, self.total)
    }
}

fn check_load_state(
    #[cfg(feature = "loading")] curr_loading_state: Res<
        State<crate::ui::loading::LoadingScreenState>,
    >,
    mut next_state: ResMut<NextState<GameState>>,
    mut loading_data: ResMut<LoadingData>,
    asset_server: Res<AssetServer>,
) {
    #[cfg(feature = "loading")]
    if !matches!(
        curr_loading_state.get(),
        crate::ui::loading::LoadingScreenState::Loading
    ) {
        return;
    }

    let (loaded, total) = loading_data.current(&asset_server);
    if loaded == total {
        next_state.set(GameState::Play);
    }
}
