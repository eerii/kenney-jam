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

const PRICE: [u32; 11] = [10, 20, 30, 40, 50, 60, 70, 80, 90, 100, 200];

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

            let upgrades = [
                (
                    22 * ATLAS_SIZE.0,
                    save_data.max_range as f32,
                    Upgrade::Range,
                ),
                (
                    22 * ATLAS_SIZE.0 + 4,
                    save_data.max_battery as f32,
                    Upgrade::Battery,
                ),
                (
                    7 * ATLAS_SIZE.0 + 34,
                    save_data.attack,
                    Upgrade::Basic,
                ),
                (
                    10 * ATLAS_SIZE.0 + 15,
                    save_data.fire as f32,
                    Upgrade::Fire,
                ),
                (
                    13 * ATLAS_SIZE.0 + 32,
                    save_data.water as f32,
                    Upgrade::Water,
                ),
                (
                    ATLAS_SIZE.0 + 3,
                    save_data.grass as f32,
                    Upgrade::Grass,
                ),
            ];

            for (index, value, typ) in upgrades {
                column.row(|row| {
                    row.button(ShopButton::Minus(typ), |button| {
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

                    row.button(ShopButton::Plus(typ), |button| {
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
                ShopButton::Play => {
                    save_data.fire_uses = save_data.fire;
                    save_data.water_uses = save_data.water;
                    save_data.grass_uses = save_data.grass;
                    save_data.battery = save_data.max_battery;  // not sure if needed
                    reset_writer.send(RestartEvent);
                },
                ShopButton::Plus(upgrade) => {
                    match upgrade {
                        Upgrade::Range => {
                            println!("Range: {}", PRICE[save_data.range_lvl]);
                            if save_data.money >= PRICE[save_data.range_lvl] {
                                save_data.money -= PRICE[save_data.range_lvl];
                                save_data.range_lvl += 1;
                                save_data.max_range += 1;
                            }
                        },
                        Upgrade::Battery => {
                            println!("Battery");
                            if save_data.money >= PRICE[save_data.battery_lvl] {
                                save_data.money -= PRICE[save_data.battery_lvl];
                                save_data.battery_lvl += 1;
                                save_data.max_battery += 25;
                            }
                        },
                        Upgrade::Basic => {
                            println!("Basic");
                            if save_data.money >= PRICE[save_data.attack_lvl] {
                                save_data.money -= PRICE[save_data.attack_lvl];
                                save_data.attack_lvl += 1;
                                save_data.attack += 0.5;
                            }
                        },
                        Upgrade::Fire => {
                            println!("Fire: {}", PRICE[save_data.fire as usize]);
                            let lvl = save_data.fire as usize;
                            if save_data.money >= PRICE[lvl] {
                                save_data.money -= PRICE[lvl];
                                save_data.fire += 1;
                            }
                        },
                        Upgrade::Water => {
                            let lvl = save_data.water as usize;
                            println!("Water: {}", PRICE[lvl]);
                            if save_data.money >= PRICE[lvl] {
                                save_data.money -= PRICE[lvl];
                                save_data.water += 1;
                            }
                        },
                        Upgrade::Grass => {
                            let lvl = save_data.grass as usize;
                            println!("Grass: {}", PRICE[lvl]);
                            if save_data.money >= PRICE[lvl] {
                                save_data.money -= PRICE[lvl];
                                save_data.grass += 1;
                            }
                        },
                    }
                },
                ShopButton::Minus(upgrade) => {
                    match upgrade {
                        Upgrade::Range => {
                            println!("Range");
                            if save_data.range_lvl > 0 {
                                save_data.range_lvl -= 1;
                                save_data.max_range -= 1;
                                save_data.money += PRICE[save_data.range_lvl];
                            }
                        },
                        Upgrade::Battery => {
                            println!("Battery");
                            if save_data.range_lvl > 0 {
                                save_data.battery_lvl -= 1;
                                save_data.max_battery -= 25;
                                save_data.money += PRICE[save_data.battery_lvl];
                            }
                        },
                        Upgrade::Basic => {
                            println!("Basic");
                            if save_data.attack_lvl > 0 {
                                save_data.attack_lvl -= 1;
                                save_data.attack -= 0.5;
                                save_data.money += PRICE[save_data.attack_lvl];
                            }
                        },
                        Upgrade::Fire => {
                            println!("Fire");
                            if save_data.fire > 0 {
                                save_data.fire -= 1;
                                save_data.money += PRICE[save_data.fire as usize];
                            }
                        },
                        Upgrade::Water => {
                            println!("Water");
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
                    }
                },
            }
        }
    }
}
