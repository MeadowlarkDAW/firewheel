use std::time::Duration;

use crate::Point;

pub use keyboard_types::{
    Code, CompositionEvent, CompositionState, Key, KeyState, KeyboardEvent, Location, Modifiers,
};

#[derive(Debug)]
pub enum InputEvent {
    Animation(AnimationEvent),
    Pointer(PointerEvent),
    PointerLocked,
    PointerUnlocked,
    Keyboard(KeyboardEvent),
    TextComposition(CompositionEvent),
    TextCompositionFocused,
    TextCompositionUnfocused,
    VisibilityShown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerButtonState {
    JustPressed,
    JustUnpressed,
    StayedUnpressed,
    StayedPressed,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointerEvent {
    pub position: Point,
    pub delta: Point,
    pub left_button: PointerButtonState,
    pub middle_button: PointerButtonState,
    pub right_button: PointerButtonState,
    pub scroll_delta_x: f32,
    pub scroll_delta_y: f32,
    pub modifiers: Modifiers,
}

impl PointerEvent {
    pub fn any_button_just_pressed(&self) -> bool {
        self.left_button == PointerButtonState::JustPressed
            || self.right_button == PointerButtonState::JustPressed
            || self.middle_button == PointerButtonState::JustPressed
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyboardEventsListen {
    None,
    TextComposition,
    Keys,
    KeysAndTextComposition,
}

impl Default for KeyboardEventsListen {
    fn default() -> Self {
        KeyboardEventsListen::None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnimationEvent {
    pub time_delta: Duration,
}
