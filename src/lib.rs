mod application;
mod background;
mod core_types;
mod message;
mod root;
mod runner;
mod wgpu_renderer;
mod widgets;

pub mod settings;
pub mod texture;

use std::hash::{Hash, Hasher as _};

pub use application::Application;
pub use background::Background;
pub use core_types::*;
pub use message::Message;
pub use root::Root;
pub use runner::Runner;
pub use settings::Settings;
pub use texture::{Texture, TextureSource};
pub use wgpu_renderer::atlas;
pub use widgets::*;

pub use baseview::{
    KeyboardEvent, MouseButton, MouseClick, MouseCursor, MouseEvent, Parent,
    WindowEvent, WindowHandle, WindowInfo,
};

pub trait IdGroup:
    std::fmt::Debug + Hash + Copy + Clone + PartialEq + Eq
{
    fn hash_to_u64(&self) -> u64 {
        let mut hasher = Hasher::default();
        self.hash(&mut hasher);
        hasher.finish()
    }
}

impl IdGroup for u64 {}

#[inline]
pub fn hash_id<ID: IdGroup>(id: ID) -> u64 {
    let mut hasher = core_types::Hasher::default();
    id.hash(&mut hasher);
    hasher.finish()
}
