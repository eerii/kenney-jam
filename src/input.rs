//! Input module

use bevy::prelude::*;
pub use leafwing_input_manager::prelude::ActionState;
use leafwing_input_manager::prelude::*;

use crate::{
    data::{Persistent, SaveData},
    PlayState,
};

// ······
// Plugin
// ······

/// Input
/// Uses the leafwing input manager for handling input
/// This allows mapping multiple sources to the same `Action`
pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<Action>::default())
            .add_systems(
                OnEnter(PlayState::default()),
                init.run_if(run_once()),
            );

        #[cfg(feature = "menu")]
        app.add_systems(
            Update,
            handle_input.in_set(crate::PlaySet::Tick),
        );
    }
}

// ··········
// Components
// ··········

/// These are all the possible actions that have an input mapping
/// CHANGE: Add player actions here and configure the default mappings in `init`
#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect)]
pub enum Action {
    /// Two axis input usually assigned to WASD or the left gamepad stick
    Move,
    /// Button press usually assigned to Escape or Start
    Pause,
    AttackRegular,
    AttackFire,
    AttackWater,
    AttackGrass,
    NextAttack,
    PreviousAttack,
}

// ·······
// Systems
// ·······

/// Create a new input manager for the general game
fn init(mut cmd: Commands) {
    let mut input_map = InputMap::default();
    input_map
        .insert(Action::Move, KeyboardVirtualDPad::WASD)
        .insert(Action::Move, GamepadStick::LEFT)
        .insert(Action::Pause, KeyCode::Escape)
        .insert(Action::Pause, GamepadButtonType::Start)
        .insert(Action::AttackRegular, KeyCode::Digit1)
        .insert(
            Action::AttackRegular,
            GamepadButtonType::DPadUp,
        )
        .insert(Action::AttackFire, KeyCode::Digit2)
        .insert(
            Action::AttackFire,
            GamepadButtonType::DPadRight,
        )
        .insert(Action::AttackWater, KeyCode::Digit3)
        .insert(
            Action::AttackWater,
            GamepadButtonType::DPadDown,
        )
        .insert(Action::AttackGrass, KeyCode::Digit4)
        .insert(
            Action::AttackGrass,
            GamepadButtonType::DPadLeft,
        )
        .insert(Action::NextAttack, KeyCode::ArrowRight)
        .insert(
            Action::NextAttack,
            GamepadButtonType::North,
        )
        .insert(
            Action::PreviousAttack,
            KeyCode::ArrowLeft,
        )
        .insert(
            Action::PreviousAttack,
            GamepadButtonType::South,
        );

    cmd.spawn(InputManagerBundle::with_map(input_map));
}

/// Read the input and perform actions
#[cfg(feature = "menu")]
fn handle_input(
    input: Query<&ActionState<Action>>,
    mut next_state: ResMut<NextState<PlayState>>,
    mut save_data: ResMut<Persistent<SaveData>>,
) {
    use crate::enemy::Element;

    let Ok(input) = input.get_single() else { return };

    if input.just_pressed(&Action::Pause) {
        next_state.set(PlayState::Menu)
    }

    if input.just_pressed(&Action::AttackRegular) {
        let _ = save_data.update(|data| data.attack_selected = Element::Basic);
    }
    if input.just_pressed(&Action::AttackFire) {
        let _ = save_data.update(|data| data.attack_selected = Element::Fire);
    }
    if input.just_pressed(&Action::AttackWater) {
        let _ = save_data.update(|data| data.attack_selected = Element::Water);
    }
    if input.just_pressed(&Action::AttackGrass) {
        let _ = save_data.update(|data| data.attack_selected = Element::Grass);
    }

    if input.just_pressed(&Action::NextAttack) {
        let _ = save_data.update(|data| data.attack_selected.next());
    }
    if input.just_pressed(&Action::PreviousAttack) {
        let _ = save_data.update(|data| data.attack_selected.prev());
    }
}
