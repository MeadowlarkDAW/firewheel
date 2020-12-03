use crate::{texture, Color, Font, HAlign, Point, Size, VAlign};

pub enum Primitive {
    Texture(Texture),
    SingleLineText(SingleLineText),
}

pub struct Texture {
    pub handle: texture::Handle,
    pub center_position: Point,
    pub rotation: f32,
}

pub struct SingleLineText {
    pub text: String,
    pub font_color: Color,
    pub font_size: f32,
    pub font_family: Font,
    pub position: Point,
    pub scissor_rect: Option<Size>,
    pub h_align: HAlign,
    pub v_align: VAlign,
}
