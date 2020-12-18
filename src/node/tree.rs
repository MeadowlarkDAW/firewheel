use super::{Handle, Loader, Node};
use std::cell::RefCell;
use std::rc::{Rc, Weak};

pub struct Tree {
    pub(crate) nodes: Vec<Rc<RefCell<dyn Node + 'static>>>,
}

impl Tree {
    pub(crate) fn new(mut node_loaders: Vec<Loader>) -> Self {
        let mut nodes: Vec<Rc<RefCell<dyn Node + 'static>>> =
            Vec::with_capacity(node_loaders.len());

        while let Some(node_loader) = node_loaders.pop() {
            nodes.push(node_loader.node);
        }

        Self { nodes }
    }
}
