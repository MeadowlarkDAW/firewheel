use super::Point;

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct Anchor {
    pub h_align: HAlign,
    pub v_align: VAlign,
    pub offset: Point,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HAlign {
    Left,
    Center,
    Right,
}

impl Default for HAlign {
    fn default() -> Self {
        HAlign::Left
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VAlign {
    Top,
    Center,
    Bottom,
}

impl Default for VAlign {
    fn default() -> Self {
        VAlign::Top
    }
}