#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct Anchor {
    pub h_align: HAlign,
    pub v_align: VAlign,
}

impl Anchor {
    pub fn new(h_align: HAlign, v_align: VAlign) -> Self {
        Self { h_align, v_align }
    }

    pub fn top_left() -> Self {
        Self {
            h_align: HAlign::Left,
            v_align: VAlign::Top,
        }
    }

    pub fn top_center() -> Self {
        Self {
            h_align: HAlign::Center,
            v_align: VAlign::Top,
        }
    }

    pub fn top_right() -> Self {
        Self {
            h_align: HAlign::Right,
            v_align: VAlign::Top,
        }
    }

    pub fn center_left() -> Self {
        Self {
            h_align: HAlign::Left,
            v_align: VAlign::Center,
        }
    }

    pub fn center() -> Self {
        Self {
            h_align: HAlign::Center,
            v_align: VAlign::Center,
        }
    }

    pub fn center_right() -> Self {
        Self {
            h_align: HAlign::Right,
            v_align: VAlign::Center,
        }
    }

    pub fn bottom_left() -> Self {
        Self {
            h_align: HAlign::Left,
            v_align: VAlign::Bottom,
        }
    }

    pub fn bottom_center() -> Self {
        Self {
            h_align: HAlign::Center,
            v_align: VAlign::Bottom,
        }
    }

    pub fn bottom_right() -> Self {
        Self {
            h_align: HAlign::Right,
            v_align: VAlign::Bottom,
        }
    }
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
