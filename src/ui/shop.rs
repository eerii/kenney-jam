use bevy::prelude::*;
use bevy_alt_ui_navigation_lite::prelude::*;
use sickle_ui::prelude::*;

use super::{
    widgets::{UiButtonWidget, UiTextWidget},
    UI_GAP,
};
use crate::{
    assets::{CoreAssets, SpriteAssets, ATLAS_SIZE},
    camera::BACKGROUND_LUMINANCE,
    data::{GameOptions, Persistent, RestartEvent, SaveData},
    ui::UiRootContainer,
    GameState, SCALE,
};

const SIZE: Val = Val::Px(16. * SCALE);

// ······
// Plugin
// ······

pub struct ShopPlugin;

impl Plugin for ShopPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Shop), init).add_systems(
            Update,
            handle_buttons.run_if(in_state(GameState::Shop)),
        );
    }
}

// ··········
// Components
// ··········

#[derive(Component)]
enum ShopButton {
    Play,
    Plus(()),
    Minus(()),
}

// ·······
// Systems
// ·······

fn init(
    mut cmd: Commands,
    root: Query<Entity, With<UiRootContainer>>,
    assets: Res<CoreAssets>,
    sprite_assets: Res<SpriteAssets>,
    options: Res<Persistent<GameOptions>>,
    save_data: Res<Persistent<SaveData>>,
) {
    let Ok(root) = root.get_single() else { return };

    cmd.ui_builder(root)
        .column(|column| {
            column
                .style()
                .width(Val::Percent(100.))
                .align_items(AlignItems::Center)
                .justify_content(JustifyContent::Center)
                .row_gap(UI_GAP);

            let upgrades = [
                (
                    22 * ATLAS_SIZE.0,
                    save_data.max_range as f32,
                ),
                (
                    22 * ATLAS_SIZE.0 + 4,
                    save_data.max_battery as f32,
                ),
                (7 * ATLAS_SIZE.0 + 34, save_data.attack),
                (10 * ATLAS_SIZE.0 + 15, save_data.fire as f32),
                (13 * ATLAS_SIZE.0 + 32, save_data.water as f32),
                (ATLAS_SIZE.0 + 3, save_data.grass as f32),
            ];

            for (index, value) in upgrades {
                column.row(|row| {
                    row.button(ShopButton::Minus(()), |button| {
                        button.text("-".into(), assets.font.clone());
                    })
                    .style()
                    .width(SIZE)
                    .height(SIZE);

                    row.spawn((
                        ImageBundle {
                            style: Style {
                                width: SIZE,
                                height: SIZE,
                                ..default()
                            },
                            image: UiImage::new(sprite_assets.one_bit.clone()),
                            ..default()
                        },
                        TextureAtlas {
                            layout: sprite_assets.one_bit_atlas.clone(),
                            index,
                        },
                    ));

                    row.text(
                        format!("{}", value),
                        assets.font.clone(),
                    );

                    row.button(ShopButton::Plus(()), |button| {
                        button.text("+".into(), assets.font.clone());
                    })
                    .style()
                    .width(SIZE)
                    .height(SIZE);

                    row.style()
                        .width(Val::Percent(80.))
                        .justify_content(JustifyContent::Center)
                        .column_gap(Val::Px(4.))
                        .column_gap(UI_GAP);
                });
            }

            let mut button = column.button(ShopButton::Play, |button| {
                button.text("Play".into(), assets.font.clone());
            });
            #[cfg(feature = "menu")]
            button.insert(Focusable::new().prioritized());
        })
        .insert(StateScoped(GameState::Shop))
        .style()
        .background_color(options.base_color.with_luminance(BACKGROUND_LUMINANCE));
}

fn handle_buttons(
    buttons: Query<&ShopButton>,
    mut nav_event_reader: EventReader<NavEvent>,
    mut reset_writer: EventWriter<RestartEvent>,
) {
    for event in nav_event_reader.read() {
        if let NavEvent::NoChanges {
            from,
            request: NavRequest::Action,
        } = event
        {
            let Ok(buttons) = buttons.get(*from.first()) else { continue };

            match buttons {
                ShopButton::Play => {
                    reset_writer.send(RestartEvent);
                },
                ShopButton::Plus(_) => {},
                ShopButton::Minus(_) => {},
            }
        }
    }
}
