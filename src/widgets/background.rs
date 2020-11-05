use crate::{Color, Point};

pub enum Background {
    SolidColor(Color),
    Texture(u64),
    MultipleTextures(Vec<(u64, Point<u16>)>),
}
