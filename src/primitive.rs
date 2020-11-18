use crate::{
    Color, Font, HorizontalAlign, IdGroup, Point, Size, VerticalAlign,
};

pub enum Primitive<TexID: IdGroup> {
    Texture(Texture<TexID>),
    SingleLineText(SingleLineText),
}

pub struct Texture<TexID: IdGroup> {
    pub texture_id: TexID,
    pub center_position: Point<u16>,
    pub rotation: f32,
}

pub struct SingleLineText {
    pub text: String,
    pub font_color: Color,
    pub font_size: f32,
    pub font_family: Font,
    pub position: Point<u16>,
    pub scissor_rect: Option<Size<u16>>,
    pub h_align: HorizontalAlign,
    pub v_align: VerticalAlign,
}
