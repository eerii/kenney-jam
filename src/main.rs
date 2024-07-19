use bevy::prelude::*;
use kenney_jam::{
    assets::SpriteAssets,
    input::{Action, ActionState},
    AppConfig, GamePlugin, GameState,
};

fn main() {
    App::new()
        .insert_resource(AppConfig::default())
        .add_plugins(GamePlugin)
        .add_systems(
            OnEnter(GameState::Play),
            init.run_if(run_once()),
        )
        .add_systems(
            Update,
            update_player.run_if(in_state(GameState::Play)),
        )
        .run();
}

// ··········
// Components
// ··········

#[derive(Component)]
struct Player;

// ·······
// Systems
// ·······

fn init(mut cmd: Commands, sprite_assets: Res<SpriteAssets>) {
    cmd.spawn((
        SpriteBundle {
            transform: Transform::from_scale(Vec3::splat(3.0)),
            texture: sprite_assets.one_bit.clone(),
            ..default()
        },
        TextureAtlas {
            layout: sprite_assets.one_bit_atlas.clone(),
            index: 24,
        },
        Player,
    ));
}

fn update_player(
    time: Res<Time>,
    input: Query<&ActionState<Action>>,
    mut player: Query<&mut Transform, With<Player>>,
) {
    let Ok(mut trans) = player.get_single_mut() else { return };
    let Ok(input) = input.get_single() else { return };

    let axis = input.clamped_axis_pair(&Action::Move);
    // TODO: Proper movement
    let dir = axis.unwrap_or_default().x();
    trans.translation.x += dir * 1000. * time.delta_seconds();
    let dir = axis.unwrap_or_default().y();
    trans.translation.y += dir * 1000. * time.delta_seconds();
}
