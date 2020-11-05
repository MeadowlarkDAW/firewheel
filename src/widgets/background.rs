use crate::Color;

pub enum Background {
    SolidColor(Color),
    Texture(u64),
}
