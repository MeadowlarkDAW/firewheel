use crate::{texture, Color};

pub enum Background<T: texture::IdGroup> {
    SolidColor(Color),
    Texture(T),
}
