use bevy::prelude::*;
use bevy_alt_ui_navigation_lite::prelude::*;
use sickle_ui::prelude::*;

use crate::{
    assets::CoreAssets,
    data::{GameOptions, Persistent, SaveData},
    ui::{
        widgets::{UiButtonWidget, UiTextWidget},
        UiRootContainer, UI_GAP,
    },
    GameState, PlayState,
};

// ······
// Plugin
// ······

pub struct ConfirmPlugin;

impl Plugin for ConfirmPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(PlayState::ToShop), confirm_shop)
            .add_systems(
                OnEnter(PlayState::ToLevel),
                confirm_level,
            )
            .add_systems(
                OnEnter(PlayState::GameWon),
                confirm_game_won,
            )
            .add_systems(
                OnEnter(PlayState::GameOver),
                confirm_game_over,
            )
            .add_systems(
                Update,
                handle_buttons.run_if(
                    in_state(PlayState::ToShop)
                        .or_else(in_state(PlayState::ToLevel))
                        .or_else(in_state(PlayState::GameWon))
                        .or_else(in_state(PlayState::GameOver)),
                ),
            );
    }
}

// ··········
// Components
// ··········

#[derive(Component)]
enum ConfirmButton {
    Shop,
    Level,
    Back,
    GameOver,
    GameWon,
}

// ·······
// Systems
// ·······

fn confirm_shop(
    mut cmd: Commands,
    root: Query<Entity, With<UiRootContainer>>,
    assets: Res<CoreAssets>,
    options: Res<Persistent<GameOptions>>,
) {
    let Ok(root) = root.get_single() else { return };

    cmd.ui_builder(root)
        .column(|base| {
            base.style()
                .width(Val::Percent(100.))
                .height(Val::Percent(100.))
                .align_items(AlignItems::Center)
                .justify_content(JustifyContent::Center);

            base.column(|column| {
                column
                    .style()
                    .width(Val::Percent(80.))
                    .height(Val::Percent(25.))
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::Center)
                    .row_gap(UI_GAP);

                column.text(
                    "Back to the shop".into(),
                    assets.font.clone(),
                );

                column.text(
                    "You will not lose gems".into(),
                    assets.font.clone(),
                );

                column.row(|row| {
                    row.style()
                        .width(Val::Percent(100.))
                        .justify_content(JustifyContent::Center)
                        .column_gap(UI_GAP);

                    row.button(ConfirmButton::Shop, |button| {
                        button.text("Confirm".into(), assets.font.clone());
                    });

                    row.button(ConfirmButton::Back, |button| {
                        button.text("Back".into(), assets.font.clone());
                    });
                });
            })
            .style()
            .background_color(options.base_color.with_luminance(0.02));
        })
        .insert(StateScoped(PlayState::ToShop));
}

// I know it's ugly to repeat this
fn confirm_level(
    mut cmd: Commands,
    root: Query<Entity, With<UiRootContainer>>,
    assets: Res<CoreAssets>,
    options: Res<Persistent<GameOptions>>,
) {
    let Ok(root) = root.get_single() else { return };

    cmd.ui_builder(root)
        .column(|base| {
            base.style()
                .width(Val::Percent(100.))
                .height(Val::Percent(100.))
                .align_items(AlignItems::Center)
                .justify_content(JustifyContent::Center);

            base.column(|column| {
                column
                    .style()
                    .width(Val::Percent(80.))
                    .height(Val::Percent(25.))
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::Center)
                    .row_gap(UI_GAP);

                column.text(
                    "Descend to the next level".into(),
                    assets.font.clone(),
                );

                column.row(|row| {
                    row.style()
                        .width(Val::Percent(100.))
                        .justify_content(JustifyContent::Center)
                        .column_gap(UI_GAP);

                    row.button(ConfirmButton::Level, |button| {
                        button.text("Confirm".into(), assets.font.clone());
                    });

                    row.button(ConfirmButton::Back, |button| {
                        button.text("Back".into(), assets.font.clone());
                    });
                });
            })
            .style()
            .background_color(options.base_color.with_luminance(0.02));
        })
        .insert(StateScoped(PlayState::ToLevel));
}

fn confirm_game_over(
    mut cmd: Commands,
    root: Query<Entity, With<UiRootContainer>>,
    assets: Res<CoreAssets>,
    options: Res<Persistent<GameOptions>>,
    mut save_data: ResMut<Persistent<SaveData>>,
) {
    let Ok(root) = root.get_single() else { return };

    cmd.ui_builder(root)
        .column(|base| {
            base.style()
                .width(Val::Percent(100.))
                .height(Val::Percent(100.))
                .align_items(AlignItems::Center)
                .justify_content(JustifyContent::Center);

            base.column(|column| {
                column
                    .style()
                    .width(Val::Percent(80.))
                    .height(Val::Percent(25.))
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::Center)
                    .row_gap(UI_GAP);

                column.text(
                    "You ran out of battery".into(),
                    assets.font.clone(),
                );

                column.text(
                    format!(
                        "You lost {} gems",
                        save_data.money - (save_data.money / 2)
                    ),
                    assets.font.clone(),
                );

                save_data.money /= 2;
                save_data.deaths += 1;

                column.row(|row| {
                    row.style()
                        .width(Val::Percent(100.))
                        .justify_content(JustifyContent::Center)
                        .column_gap(UI_GAP);

                    row.button(ConfirmButton::GameOver, |button| {
                        button.text(
                            "Back to shop".into(),
                            assets.font.clone(),
                        );
                    })
                    .style()
                    .width(Val::Px(350.));
                });
            })
            .style()
            .background_color(options.base_color.with_luminance(0.02));
        })
        .insert(StateScoped(PlayState::GameOver));
}

fn confirm_game_won(
    mut cmd: Commands,
    root: Query<Entity, With<UiRootContainer>>,
    assets: Res<CoreAssets>,
    options: Res<Persistent<GameOptions>>,
    save_data: Res<Persistent<SaveData>>,
) {
    let Ok(root) = root.get_single() else { return };

    cmd.ui_builder(root)
        .column(|base| {
            base.style()
                .width(Val::Percent(100.))
                .height(Val::Percent(100.))
                .align_items(AlignItems::Center)
                .justify_content(JustifyContent::Center);

            base.column(|column| {
                column
                    .style()
                    .width(Val::Percent(80.))
                    .height(Val::Percent(60.))
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::Center)
                    .row_gap(UI_GAP);

                column.text(
                    "You got the artifact".into(),
                    assets.font.clone(),
                );

                column.text(
                    if save_data.enemies_killed == 0 {
                        "without harming anybody".into()
                    } else {
                        format!(
                            "slaughtering {} animals",
                            save_data.enemies_killed
                        )
                    },
                    assets.font.clone(),
                );

                column.text(
                    format!(
                        "completing {} levels",
                        save_data.levels_completed,
                    ),
                    assets.font.clone(),
                );

                column.text(
                    format!(
                        "and perishing {} times",
                        save_data.deaths,
                    ),
                    assets.font.clone(),
                );

                column.text(
                    format!(
                        "score {}",
                        (save_data.levels_completed as i32 + 1)
                            * save_data.enemies_killed as i32
                            * 100
                            - save_data.deaths as i32 * 200
                    ),
                    assets.font.clone(),
                );

                column.row(|row| {
                    row.style()
                        .width(Val::Percent(100.))
                        .justify_content(JustifyContent::Center)
                        .column_gap(UI_GAP);

                    row.button(ConfirmButton::GameWon, |button| {
                        button.text("Try again".into(), assets.font.clone());
                    })
                    .style()
                    .width(Val::Px(350.));
                });
            })
            .style()
            .background_color(options.base_color.with_luminance(0.02));
        })
        .insert(StateScoped(PlayState::GameWon));
}

fn handle_buttons(
    buttons: Query<&ConfirmButton>,
    mut nav_event_reader: EventReader<NavEvent>,
    mut next_state: ResMut<NextState<GameState>>,
    mut next_play_state: ResMut<NextState<PlayState>>,
    mut save_data: ResMut<Persistent<SaveData>>,
) {
    for event in nav_event_reader.read() {
        if let NavEvent::NoChanges {
            from,
            request: NavRequest::Action,
        } = event
        {
            let Ok(buttons) = buttons.get(*from.first()) else { continue };

            match buttons {
                ConfirmButton::Shop => next_state.set(GameState::Shop),
                ConfirmButton::Level => {
                    next_state.set(GameState::LevelTransition);
                    save_data.levels_completed += 1;
                },
                ConfirmButton::Back => next_play_state.set(PlayState::Play),
                ConfirmButton::GameOver => {
                    next_state.set(GameState::Shop);
                },
                ConfirmButton::GameWon => {
                    next_state.set(GameState::End);
                },
            }
        }
    }
}
