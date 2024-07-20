use bevy::prelude::*;
use sickle_ui::prelude::*;

use crate::{
    assets::{SpriteAssets, ATLAS_SIZE},
    data::{GameOptions, Persistent, SaveData},
    ui::UiRootContainer,
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
}

#[derive(Component)]
struct Display {
    index: usize,
    display: DisplayType,
}

// ·······
// Systems
// ·······

fn init(
    mut cmd: Commands,
    root: Query<Entity, With<UiRootContainer>>,
    sprite_assets: Res<SpriteAssets>,
    options: Res<Persistent<GameOptions>>,
) {
    let Ok(root) = root.get_single() else { return };

    let displays = [
        Display {
            index: 22 * ATLAS_SIZE.0,
            display: DisplayType::Connection,
        },
        Display {
            index: 22 * ATLAS_SIZE.0 + 4,
            display: DisplayType::Battery,
        },
    ];

    cmd.ui_builder(root)
        .column(|column| {
            column
                .style()
                .width(Val::Px(24. * SCALE))
                .align_items(AlignItems::Center)
                .justify_content(JustifyContent::Center)
                .row_gap(Val::Px(12.));

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
        })
        .insert(StateScoped(PlayState::Play))
        .style()
        .background_color(options.base_color.with_luminance(0.02));
}

fn update_displays(
    mut displays: Query<(&mut TextureAtlas, &Display)>,
    save_data: Res<Persistent<SaveData>>,
) {
    for (mut atlas, display) in displays.iter_mut() {
        let percent = match display.display {
            DisplayType::Connection => save_data.level as f32 / save_data.max_range as f32,
            DisplayType::Battery => 1. - save_data.battery as f32 / save_data.max_battery as f32,
        };
        let offset = (percent.clamp(0., 0.99) * 4.).floor() as usize;
        atlas.index = display.index + offset;
    }
}
