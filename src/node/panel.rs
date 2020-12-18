use super::{Handle, Loader, Node};
use crate::{Point, Primitive, Rect, Size};
use std::cell::RefCell;
use std::rc::Rc;

pub struct Panel;

impl Panel {
    pub fn new(handle: &mut Handle<Self>) -> Loader {
        let panel = Panel {};

        let node = Rc::new(RefCell::new(panel));
        handle.node = Rc::downgrade(&node);
        Loader::new(node)
    }
}

impl Node for Panel {
    fn render_bounds(&self) -> Rect {
        Rect::new(Point::ORIGIN, Size::new(0.0, 0.0))
    }

    fn primitives<'a>(&mut self) -> &'a [&'a Primitive] {
        &[]
    }
}
