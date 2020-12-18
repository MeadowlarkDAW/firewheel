use crate::{Primitive, Rect};
use std::cell::RefCell;
use std::rc::{Rc, Weak};

mod panel;
mod tree;
mod v_slider;

pub use panel::Panel;

pub(crate) use tree::*;

pub struct Handle<N: Node> {
    pub(crate) node: Weak<RefCell<N>>,
}

impl<N: Node> Handle<N> {
    pub fn new() -> Self {
        Self { node: Weak::new() }
    }
}

impl<N: Node> Default for Handle<N> {
    fn default() -> Self {
        Self { node: Weak::new() }
    }
}

impl<N: Node> Clone for Handle<N> {
    fn clone(&self) -> Self {
        Handle {
            node: self.node.clone(),
        }
    }
}

pub struct Loader {
    pub(crate) node: Rc<RefCell<dyn Node + 'static>>,
}

impl Loader {
    pub fn new(node: Rc<RefCell<dyn Node + 'static>>) -> Self {
        Self { node }
    }
}

pub trait Node {
    fn render_bounds(&self) -> Rect;

    fn primitives<'a>(&mut self) -> &'a [&'a Primitive];
}
