use super::{Size, Point};

pub trait Widget {
    fn wants_mouse_events(&self) -> bool {
        false
    }

    fn wants_keyboard_events(&self) -> bool {
        false
    }

    fn has_animation(&self) -> bool {
        false
    }

    fn current_size(&self) -> Size {
        Size::default()
    }
}