use crate::node;

pub trait Application: Sized {
    fn load_nodes(&mut self) -> Vec<node::Loader>;
}
