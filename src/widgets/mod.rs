use crate::{Primitive, Rect};

mod v_slider;

pub trait Widget {
    fn needs_redraw(&self) -> bool;

    fn render_bounds(&self) -> Rect;

    fn primitives<'a>(&self) -> &'a [&'a Primitive];
}
