use crate::{Color, IdGroup};

pub enum Background<ID: IdGroup> {
    SolidColor(Color),
    Texture(ID),
}
