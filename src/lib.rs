mod application;
mod background;
mod core_types;
mod message;
mod root;
mod runner;
mod wgpu_renderer;
mod widgets;

pub mod primitive;
pub mod settings;
pub mod texture;

pub use application::Application;
pub use background::Background;
pub use core_types::*;
pub use message::Message;
pub use primitive::Primitive;
pub use root::Root;
pub use runner::Runner;
pub use settings::Settings;
pub use texture::{Texture, TextureSource};
pub use wgpu_renderer::atlas;
pub use widgets::*;

pub use baseview::{
    MouseButton, MouseClick, MouseCursor, MouseEvent, Parent, WindowEvent,
    WindowHandle, WindowInfo,
};

pub use wgpu_glyph::{HorizontalAlign, VerticalAlign};

pub trait IdGroup:
    std::fmt::Debug + std::hash::Hash + Copy + Clone + PartialEq + Eq + 'static
{
}
