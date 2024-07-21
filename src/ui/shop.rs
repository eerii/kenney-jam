use bevy::prelude::*;
use bevy_alt_ui_navigation_lite::prelude::*;
use sickle_ui::prelude::*;

use crate::{
    assets::{CoreAssets, SpriteAssets, ATLAS_SIZE},
    camera::BACKGROUND_LUMINANCE,
    data::{GameOptions, Persistent, RestartEvent, SaveData},
    ui::{
        menu::navigation::on_mouse_move,
        widgets::{UiButtonWidget, UiTextWidget},
        UiRootContainer, UI_GAP,
    },
    GameState, SCALE,
};

const SIZE: Val = Val::Px(16. * SCALE);

const PRICE: [u32; 11] = [10, 20, 30, 40, 50, 60, 70, 80, 90, 100, 999];

// ······
// Plugin
// ······

pub struct ShopPlugin;

impl Plugin for ShopPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Shop), init)
            .add_systems(
                PreUpdate,
                on_mouse_move
                    .run_if(state_changed::<GameState>.and_then(in_state(GameState::Shop))),
            )
            .add_systems(
                Update,
                handle_buttons.run_if(in_state(GameState::Shop)),
            )
            .add_systems(
                OnEnter(GameState::UpdateShop),
                back_to_shop,
            );
    }
}

// ··········
// Components
// ··········

#[derive(Component)]
enum ShopButton {
    Play,
    Plus(Upgrade),
    Minus(Upgrade),
}

#[derive(Clone, Copy)]
enum Upgrade {
    Range,
    Battery,
    Basic,
    Fire,
    Water,
    Grass,
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

            column.row(|row| {
                row.style()
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::Center)
                    .column_gap(UI_GAP);

                row.spawn((
                    ImageBundle {
                        style: Style {
                            width: Val::Px(16. * SCALE),
                            height: Val::Px(16. * SCALE),
                            ..default()
                        },
                        image: UiImage::new(sprite_assets.one_bit.clone()),
                        ..default()
                    },
                    TextureAtlas {
                        layout: sprite_assets.one_bit_atlas.clone(),
                        index: 10 * ATLAS_SIZE.0 + 33,
                    },
                ));

                row.text(
                    format!("{}", save_data.money),
                    assets.font.clone(),
                );
            });

            column.row(|row| {
                row.style()
                    .width(Val::Percent(80.))
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::Center)
                    .column_gap(UI_GAP);

                let upgrades = [
                    (
                        22 * ATLAS_SIZE.0,
                        save_data.range_level,
                        Upgrade::Range,
                    ),
                    (
                        22 * ATLAS_SIZE.0 + 4,
                        save_data.battery_level,
                        Upgrade::Battery,
                    ),
                    (
                        7 * ATLAS_SIZE.0 + 34,
                        save_data.attack_level,
                        Upgrade::Basic,
                    ),
                    (
                        10 * ATLAS_SIZE.0 + 15,
                        save_data.fire as usize,
                        Upgrade::Fire,
                    ),
                    (
                        13 * ATLAS_SIZE.0 + 32,
                        save_data.water as usize,
                        Upgrade::Water,
                    ),
                    (
                        ATLAS_SIZE.0 + 3,
                        save_data.grass as usize,
                        Upgrade::Grass,
                    ),
                ];

                let mut col = row.column(|column| {
                    column
                        .style()
                        .align_items(AlignItems::Center)
                        .justify_content(JustifyContent::Center)
                        .row_gap(UI_GAP)
                        .flex_grow(1.);

                    column.text("Upgrades".into(), assets.font.clone());
                });

                for (index, value, typ) in &upgrades[0..3] {
                    shop_row(
                        &mut col,
                        assets.font.clone(),
                        &sprite_assets,
                        *index,
                        *value,
                        typ,
                    );
                }

                let mut col = row.column(|column| {
                    column
                        .style()
                        .align_items(AlignItems::Center)
                        .justify_content(JustifyContent::Center)
                        .row_gap(UI_GAP)
                        .flex_grow(1.);

                    column.text("Attacks".into(), assets.font.clone());
                });

                for (index, value, typ) in &upgrades[3..6] {
                    shop_row(
                        &mut col,
                        assets.font.clone(),
                        &sprite_assets,
                        *index,
                        *value,
                        typ,
                    );
                }
            });

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
    mut save_data: ResMut<Persistent<SaveData>>,
    mut next_state: ResMut<NextState<GameState>>,
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
                    continue;
                },
                ShopButton::Plus(upgrade) => match upgrade {
                    Upgrade::Range => {
                        if save_data.range_level < 10
                            && save_data.money >= PRICE[save_data.range_level]
                        {
                            save_data.money -= PRICE[save_data.range_level];
                            save_data.range_level += 1;
                        }
                    },
                    Upgrade::Battery => {
                        if save_data.battery_level < 10
                            && save_data.money >= PRICE[save_data.battery_level]
                        {
                            save_data.money -= PRICE[save_data.battery_level];
                            save_data.battery_level += 1;
                        }
                    },
                    Upgrade::Basic => {
                        if save_data.attack_level < 10
                            && save_data.money >= PRICE[save_data.attack_level]
                        {
                            save_data.money -= PRICE[save_data.attack_level];
                            save_data.attack_level += 1;
                        }
                    },
                    Upgrade::Fire => {
                        let lvl = save_data.fire as usize;
                        if save_data.money >= PRICE[lvl] {
                            save_data.money -= PRICE[lvl];
                            save_data.fire += 1;
                        }
                    },
                    Upgrade::Water => {
                        let lvl = save_data.water as usize;
                        if save_data.money >= PRICE[lvl] {
                            save_data.money -= PRICE[lvl];
                            save_data.water += 1;
                        }
                    },
                    Upgrade::Grass => {
                        let lvl = save_data.grass as usize;
                        if save_data.money >= PRICE[lvl] {
                            save_data.money -= PRICE[lvl];
                            save_data.grass += 1;
                        }
                    },
                },
                ShopButton::Minus(upgrade) => match upgrade {
                    Upgrade::Range => {
                        if save_data.range_level > 0 {
                            save_data.range_level -= 1;
                            save_data.money += PRICE[save_data.range_level];
                        }
                    },
                    Upgrade::Battery => {
                        if save_data.battery_level > 0 {
                            save_data.battery_level -= 1;
                            save_data.money += PRICE[save_data.battery_level];
                        }
                    },
                    Upgrade::Basic => {
                        if save_data.attack_level > 0 {
                            save_data.attack_level -= 1;
                            save_data.money += PRICE[save_data.attack_level];
                        }
                    },
                    Upgrade::Fire => {
                        if save_data.fire > 0 {
                            save_data.fire -= 1;
                            save_data.money += PRICE[save_data.fire as usize];
                        }
                    },
                    Upgrade::Water => {
                        if save_data.water > 0 {
                            save_data.water -= 1;
                            save_data.money += PRICE[save_data.water as usize];
                        }
                    },
                    Upgrade::Grass => {
                        println!("Grass");
                        if save_data.grass > 0 {
                            save_data.grass -= 1;
                            save_data.money += PRICE[save_data.grass as usize];
                        }
                    },
                },
            }

            let _ = save_data.persist();
            next_state.set(GameState::UpdateShop);
        }
    }
}

fn back_to_shop(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::Shop);
}

fn shop_row(
    col: &mut UiBuilder<Entity>,
    font: Handle<Font>,
    sprite_assets: &SpriteAssets,
    index: usize,
    value: usize,
    typ: &Upgrade,
) {
    col.row(|row| {
        let mut button = row.button(ShopButton::Minus(*typ), |button| {
            let text = value
                .checked_sub(1)
                .map(|v| PRICE[v])
                .unwrap_or(0)
                .to_string();
            button.column(|column| {
                column
                    .style()
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::Center)
                    .row_gap(Val::Px(8.));
                column.text("-".into(), font.clone());
                if value > 0 {
                    column.spawn((
                        #[cfg(feature = "tts")]
                        crate::ui::tts::SpeechTag(text.clone()),
                        TextBundle::from_section(text, TextStyle {
                            font: font.clone(),
                            font_size: 14.,
                            color: Srgba::rgb(0.749, 0.475, 0.345).into(),
                        }),
                    ));
                }
            });
        });

        button.style().width(SIZE).height(SIZE);

        if value == 0 {
            button.insert((
                BackgroundColor::from(Srgba::new(0.141, 0.118, 0.118, 1.)),
                Focusable::new().blocked(),
            ));
        }

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

        row.text(format!("{}", value), font.clone());

        let mut button = row.button(ShopButton::Plus(*typ), |button| {
            button.column(|column| {
                column
                    .style()
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::Center)
                    .row_gap(Val::Px(8.));
                column.text("+".into(), font.clone());
                if value < 10 {
                    let text = format!("{}", PRICE[value]);
                    column.spawn((
                        #[cfg(feature = "tts")]
                        crate::ui::tts::SpeechTag(text.clone()),
                        TextBundle::from_section(text, TextStyle {
                            font,
                            font_size: 14.,
                            color: Srgba::rgb(0.749, 0.475, 0.345).into(),
                        }),
                    ));
                }
            });
        });

        button.style().width(SIZE).height(SIZE);

        if value >= 10 {
            button.insert((
                BackgroundColor::from(Srgba::new(0.141, 0.118, 0.118, 1.)),
                Focusable::new().blocked(),
            ));
        }

        row.style()
            .width(Val::Percent(80.))
            .justify_content(JustifyContent::Center)
            .column_gap(Val::Px(4.))
            .column_gap(UI_GAP);
    });
}
