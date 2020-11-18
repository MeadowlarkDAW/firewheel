use crate::{Color, IdGroup, Point};

pub enum Background<ID: IdGroup> {
    SolidColor(Color),
    Texture(ID),
    MultipleTextures(Vec<(ID, Point<u16>)>),
}
