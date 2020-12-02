#[derive(Debug, Copy, Clone, PartialEq)]
pub enum DrawState {
    DrawNormal,
    DrawFocus,
    DrawHover,
    DrawActive,
    DrawStateCount,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Visibility {
    FullyObscured,
    PartiallyObscured,
    Unobscured,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum HAlign {
    Left,
    Center,
    Right,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum VAlign {
    Top,
    Middle,
    Bottom,
}

#[derive(Debug, Copy, Clone)]
pub struct Alignment {
    pub horizontal: HAlign,
    pub vertical: VAlign,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ChildAddLocation {
    AddHead,
    AddTail,
}
