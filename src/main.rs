use bevy::prelude::*;
use kenney_jam::{AppConfig, GamePlugin};

fn main() {
    App::new()
        .insert_resource(AppConfig::default())
        .add_plugins(GamePlugin)
        .run();
}
