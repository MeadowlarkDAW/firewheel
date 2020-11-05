mod application;
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

#[inline]
pub fn hash_id<ID: Hash + Copy + Clone>(id: ID) -> u64 {
    let mut hasher = core_types::Hasher::default();
    id.hash(&mut hasher);
    hasher.finish()
}
