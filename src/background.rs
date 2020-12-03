use crate::{texture, Color};

pub enum Background {
    SolidColor(Color),
    Texture(texture::Handle),
}
