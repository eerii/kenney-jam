//! Widgets module
//! This extends sickle ui with custom widgets
//! It is very helpful to reduce verbosity and to group elements together easily
//! Creating a widget is done by extending UiBuilder with new custom traits,
//! providing the functions we define as chaining options for our components

use bevy::prelude::*;
use sickle_ui::prelude::*;

const BUTTON_WIDTH: Val = Val::Px(256.);
const BUTTON_HEIGHT: Val = Val::Px(64.);

const FONT_SIZE_TEXT: f32 = 20.;
const FONT_SIZE_TITLE: f32 = 28.;

/// Base color for UI buttons
pub const BUTTON_COLOR: Color = Color::srgb(0.478, 0.267, 0.29);

/// Text color
pub const TEXT_COLOR: Color = Color::srgb(0.812, 0.776, 0.722);

// ······
// Traits
// ······

/// Creates a text bundle with custom styling for titles and text
pub trait UiTextWidget {
    /// Append a text ui element
    fn text(&mut self, text: String, font: Handle<Font>) -> UiBuilder<Entity>;
    /// Append a text ui element, formatted like a title
    fn title(&mut self, text: String, font: Handle<Font>) -> UiBuilder<Entity>;
}

impl UiTextWidget for UiBuilder<'_, Entity> {
    fn text(&mut self, text: String, font: Handle<Font>) -> UiBuilder<Entity> {
        self.spawn((
            TextBundle::from_section(text.clone(), TextStyle {
                font,
                font_size: FONT_SIZE_TEXT,
                color: TEXT_COLOR,
            }),
            #[cfg(feature = "tts")]
            super::tts::SpeechTag(text),
        ))
    }

    fn title(&mut self, text: String, font: Handle<Font>) -> UiBuilder<Entity> {
        self.spawn((
            TextBundle::from_section(text.clone(), TextStyle {
                font,
                font_size: FONT_SIZE_TITLE,
                color: TEXT_COLOR,
            }),
            #[cfg(feature = "tts")]
            super::tts::SpeechTag(text),
        ))
    }
}

/// Creates an image bundle
pub trait UiImageWidget {
    /// Append an image element
    fn image(&mut self, image: Handle<Image>) -> UiBuilder<Entity>;
}

impl UiImageWidget for UiBuilder<'_, Entity> {
    fn image(&mut self, image: Handle<Image>) -> UiBuilder<Entity> {
        self.spawn(ImageBundle {
            image: UiImage::new(image),
            ..default()
        })
    }
}

/// Creates a "button"
/// This is not a real bevy ui button if we are using custom navigation to avoid
/// issues with interactible parts
/// To add text, you can chain one of the text widgets we added before
pub trait UiButtonWidget {
    /// Append a button ui element
    fn button<T: Component>(
        &mut self,
        component: T,
        spawn_children: impl FnOnce(&mut UiBuilder<Entity>),
    ) -> UiBuilder<Entity>;
}

#[cfg(not(feature = "menu"))]
type ButtonType = ButtonBundle;
#[cfg(feature = "menu")]
type ButtonType = NodeBundle;

impl UiButtonWidget for UiBuilder<'_, Entity> {
    fn button<T: Component>(
        &mut self,
        component: T,
        spawn_children: impl FnOnce(&mut UiBuilder<Entity>),
    ) -> UiBuilder<Entity> {
        self.container(
            (
                ButtonType {
                    style: Style {
                        width: BUTTON_WIDTH,
                        height: BUTTON_HEIGHT,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        border: UiRect::all(Val::Px(4.0)),
                        ..default()
                    },
                    background_color: BUTTON_COLOR.into(),
                    // border_radius: BorderRadius::MAX,
                    ..default()
                },
                #[cfg(feature = "menu")]
                bevy_alt_ui_navigation_lite::prelude::Focusable::default(),
                component,
            ),
            spawn_children,
        )
    }
}

/// Creates an option row
/// It consist of a name to the left and anything to the left
pub trait UiOptionRowWidget {
    /// Append an option row element
    fn option_row<T: Component>(
        &mut self,
        component: T,
        text: String,
        font: Handle<Font>,
    ) -> UiBuilder<Entity>;

    /// Option row button
    fn option_button(
        &mut self,
        spawn_children: impl FnOnce(&mut UiBuilder<Entity>),
    ) -> UiBuilder<Entity>;
}

impl UiOptionRowWidget for UiBuilder<'_, Entity> {
    fn option_row<T: Component>(
        &mut self,
        component: T,
        text: String,
        font: Handle<Font>,
    ) -> UiBuilder<Entity> {
        self.row(|row| {
            row.style()
                .width(Val::Percent(80.))
                .justify_content(JustifyContent::Center)
                .column_gap(Val::Px(4.));

            row.text(text, font).style().flex_grow(1.);

            row.insert(component);

            #[cfg(feature = "menu")]
            row.insert((
                bevy_alt_ui_navigation_lite::prelude::Focusable::default(),
                super::menu::navigation::HightlightChild,
            ));
        })
    }

    fn option_button(
        &mut self,
        spawn_children: impl FnOnce(&mut UiBuilder<Entity>),
    ) -> UiBuilder<Entity> {
        self.container(
            ButtonType {
                style: Style {
                    width: BUTTON_WIDTH,
                    height: BUTTON_HEIGHT,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    border: UiRect::all(Val::Px(6.0)),
                    ..default()
                },
                background_color: BUTTON_COLOR.into(),
                // border_radius: BorderRadius::MAX,
                ..default()
            },
            spawn_children,
        )
    }
}
