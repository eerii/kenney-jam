use bevy::prelude::*;
use sickle_ui::prelude::*;

use super::UI_GAP;
use crate::{
    assets::{CoreAssets, SpriteAssets, ATLAS_SIZE},
    data::{GameOptions, Persistent, SaveData},
    enemy::Element,
    ui::{widgets::UiTextWidget, UiRootContainer},
    PlaySet, PlayState, SCALE,
};

// ······
// Plugin
// ······

pub struct GuiPlugin;

impl Plugin for GuiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(PlayState::Play),
            (init, update_displays).chain(),
        )
        .add_systems(
            Update,
            update_displays
                .in_set(PlaySet::Animation)
                .run_if(resource_changed::<Persistent<SaveData>>),
        );
    }
}

// ··········
// Components
// ··········

enum DisplayType {
    Connection,
    Battery,
    Attack(Element),
}

#[derive(Component)]
struct Display {
    index: usize,
    display: DisplayType,
    data: Option<u32>,
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

    let displays = [
        Display {
            index: 22 * ATLAS_SIZE.0,
            display: DisplayType::Connection,
            data: None,
        },
        Display {
            index: 22 * ATLAS_SIZE.0 + 4,
            display: DisplayType::Battery,
            data: None,
        },
    ];

    let attacks = [
        Display {
            index: 7 * ATLAS_SIZE.0 + 34,
            display: DisplayType::Attack(Element::Basic),
            data: None,
        },
        Display {
            index: 10 * ATLAS_SIZE.0 + 15,
            display: DisplayType::Attack(Element::Fire),
            data: Some(save_data.fire_uses),
        },
        Display {
            index: 13 * ATLAS_SIZE.0 + 32,
            display: DisplayType::Attack(Element::Water),
            data: Some(save_data.water_uses),
        },
        Display {
            index: ATLAS_SIZE.0 + 3,
            display: DisplayType::Attack(Element::Grass),
            data: Some(save_data.grass_uses),
        },
    ];

    cmd.ui_builder(root)
        .column(|column| {
            column
                .style()
                .width(Val::Px(24. * SCALE))
                .align_items(AlignItems::Center)
                .justify_content(JustifyContent::Center)
                .row_gap(UI_GAP);

            for display in displays {
                column.spawn((
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
                        index: display.index,
                    },
                    display,
                ));
            }

            column.title(
                format!("{}", save_data.level + 1),
                assets.font.clone(),
            );
        })
        .insert(StateScoped(PlayState::Play))
        .style()
        .background_color(options.base_color.with_luminance(0.02));

    cmd.ui_builder(root)
        .column(|column| {
            column.style().flex_grow(1.);
        })
        .insert(StateScoped(PlayState::Play))
        .style();

    cmd.ui_builder(root)
        .column(|column| {
            column
                .style()
                .width(Val::Px(24. * SCALE))
                .align_items(AlignItems::Center)
                .justify_content(JustifyContent::Center)
                .row_gap(UI_GAP);

            for att in attacks {
                let data = att.data;
                column.spawn((
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
                        index: att.index,
                    },
                    att,
                ));
                if let Some(data) = data {
                    column.text(format!("{}", data), assets.font.clone());
                }
            }
        })
        .insert(StateScoped(PlayState::Play))
        .style()
        .background_color(options.base_color.with_luminance(0.02));
}

fn update_displays(
    mut displays: Query<(
        &mut TextureAtlas,
        Option<&mut BackgroundColor>,
        &Display,
    )>,
    save_data: Res<Persistent<SaveData>>,
) {
    for (mut atlas, background, display) in displays.iter_mut() {
        let percent = match &display.display {
            DisplayType::Connection => save_data.level as f32 / save_data.max_range as f32,
            DisplayType::Battery => 1. - save_data.battery as f32 / save_data.max_battery as f32,
            DisplayType::Attack(ref element) => {
                if let Some(mut color) = background {
                    let selected = *element == save_data.attack_selected;
                    *color = if selected {
                        Srgba::new(0.478, 0.267, 0.29, 1.).into()
                    } else {
                        Srgba::new(0.141, 0.118, 0.118, 1.).into()
                    };
                }
                0.
            },
        };
        let offset = (percent.clamp(0., 0.99) * 4.).floor() as usize;
        atlas.index = display.index + offset;
    }
}
