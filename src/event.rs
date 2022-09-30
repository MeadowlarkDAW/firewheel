use std::time::Duration;

use crate::{layer::LayerID, Point, Size};

pub use keyboard_types::{
    Code, CompositionEvent, CompositionState, Key, KeyState, KeyboardEvent, Location, Modifiers,
};

pub enum Event<MSG> {
    User(Box<MSG>),
    Animation(AnimationEvent),
    Mouse(MouseEvent),
    LockedMouse(LockedMouseEvent),
    Keyboard(KeyboardEvent),
    TextComposition(CompositionEvent),
    VisibilityChanged { visible: bool },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButtonState {
    JustPressed,
    JustUnpressed,
    StayedUnpressed,
    StayedPressed,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MouseEvent {
    pub layer: LayerID,
    pub position: Point,
    pub previous_position: Point,
    pub left_button: MouseButtonState,
    pub middle_button: MouseButtonState,
    pub right_button: MouseButtonState,
    pub scroll_delta_x: f32,
    pub scroll_delta_y: f32,
    pub modifiers: Modifiers,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LockedMouseEvent {
    pub delta: Point,
    pub left_button: MouseButtonState,
    pub middle_button: MouseButtonState,
    pub right_button: MouseButtonState,
    pub scroll_delta_x: f32,
    pub scroll_delta_y: f32,
    pub modifiers: Modifiers,
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DrawRegionModifiedEvent {
    pub layer: LayerID,
    pub position: Point,
    pub prev_position: Point,
    pub size: Size,
    pub prev_size: Size,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnimationEvent {
    pub time_delta: Duration,
}
