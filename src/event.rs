use std::time::Duration;

use crate::{Point, ScaleFactor};

pub use keyboard_types::{
    Code, CompositionEvent, CompositionState, Key, KeyState, KeyboardEvent, Location, Modifiers,
};

#[cfg(feature = "winit")]
pub fn from_winit_modifiers(modifiers: &winit::event::ModifiersState) -> Modifiers {
    let mut m = Modifiers::empty();

    if modifiers.contains(winit::event::ModifiersState::SHIFT) {
        m.insert(Modifiers::SHIFT);
    }
    if modifiers.contains(winit::event::ModifiersState::CTRL) {
        m.insert(Modifiers::CONTROL);
    }
    if modifiers.contains(winit::event::ModifiersState::ALT) {
        m.insert(Modifiers::ALT);
    }
    if modifiers.contains(winit::event::ModifiersState::LOGO) {
        m.insert(Modifiers::META);
    }

    m
}

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
    StayedUnpressed,
    StayedPressed,
    JustPressed,
    JustUnpressed,
}

impl PointerButtonState {
    pub fn just_pressed(&self) -> bool {
        *self == PointerButtonState::JustPressed
    }

    pub fn just_unpressed(&self) -> bool {
        *self == PointerButtonState::JustUnpressed
    }

    pub fn is_pressed(&self) -> bool {
        *self == PointerButtonState::JustPressed || *self == PointerButtonState::StayedPressed
    }

    pub fn is_unpressed(&self) -> bool {
        *self == PointerButtonState::JustUnpressed || *self == PointerButtonState::StayedUnpressed
    }
}

impl Default for PointerButtonState {
    fn default() -> Self {
        PointerButtonState::StayedUnpressed
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
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

    #[cfg(feature = "winit")]
    pub fn update_from_winit_cursor_moved(
        &mut self,
        position: winit::dpi::PhysicalPosition<f64>,
        scale_factor: ScaleFactor,
    ) {
        self.scroll_delta_x = 0.0;
        self.scroll_delta_y = 0.0;

        let new_pos = Point::new(
            position.x / scale_factor.as_f64(),
            position.y / scale_factor.as_f64(),
        );

        self.delta = new_pos - self.position;
        self.position = new_pos;
    }

    #[cfg(feature = "winit")]
    pub fn update_from_winit_mouse_input(
        &mut self,
        state: &winit::event::ElementState,
        button: &winit::event::MouseButton,
    ) {
        self.scroll_delta_x = 0.0;
        self.scroll_delta_y = 0.0;

        let is_down = *state == winit::event::ElementState::Pressed;

        let handle_button = |button_state: &mut PointerButtonState| match button_state {
            PointerButtonState::StayedUnpressed => {
                if is_down {
                    *button_state = PointerButtonState::JustPressed;
                }
            }
            PointerButtonState::StayedPressed => {
                if !is_down {
                    *button_state = PointerButtonState::JustUnpressed;
                }
            }
            PointerButtonState::JustPressed => {
                if is_down {
                    *button_state = PointerButtonState::StayedPressed;
                } else {
                    *button_state = PointerButtonState::JustUnpressed;
                }
            }
            PointerButtonState::JustUnpressed => {
                if is_down {
                    *button_state = PointerButtonState::JustPressed;
                } else {
                    *button_state = PointerButtonState::StayedUnpressed;
                }
            }
        };

        match button {
            winit::event::MouseButton::Left => handle_button(&mut self.left_button),
            winit::event::MouseButton::Middle => handle_button(&mut self.middle_button),
            winit::event::MouseButton::Right => handle_button(&mut self.right_button),
            _ => (),
        }
    }

    #[cfg(feature = "winit")]
    pub fn update_from_winit_mouse_wheel(
        &mut self,
        delta: &winit::event::MouseScrollDelta,
        _phase: &winit::event::TouchPhase,
        scale_factor: ScaleFactor,
    ) {
        const PIXELS_PER_LINE: f32 = 12.0;

        self.scroll_delta_x = 0.0;
        self.scroll_delta_y = 0.0;

        match delta {
            winit::event::MouseScrollDelta::LineDelta(x, y) => {
                self.scroll_delta_x = *x * PIXELS_PER_LINE / scale_factor.as_f32();
                self.scroll_delta_y = *y * PIXELS_PER_LINE / scale_factor.as_f32();
            }
            winit::event::MouseScrollDelta::PixelDelta(delta) => {
                self.scroll_delta_x = delta.x as f32 / scale_factor.as_f32();
                self.scroll_delta_y = delta.y as f32 / scale_factor.as_f32();
            }
        }
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
