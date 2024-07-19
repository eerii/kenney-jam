use bevy::{prelude::*, window::WindowResolution};
use kenney_jam::{
    assets::SpriteAssets,
    input::{Action, ActionState},
    AppConfig, GamePlugin, GameState,
};

const GAME_RES: Vec2 = Vec2::new(256., 192.);
const SCALE: f32 = 3.;

fn main() {
    App::new()
        .insert_resource(AppConfig {
            game_title: "Kenney Jam",
            initial_window_res: (GAME_RES * SCALE).into(),
            initial_game_res: GAME_RES,
        })
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
            transform: Transform::from_scale(Vec3::splat(SCALE as f32)),
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

    if input.just_pressed(&Action::Move) {
        let Some(axis) = input.clamped_axis_pair(&Action::Move) else { return };
        if axis.x().abs() > axis.y().abs() {
            trans.translation.x += axis.x().signum() * SCALE * 16.;
        } else {
            trans.translation.y += axis.y().signum() * SCALE * 16.;
        }
    };
}
