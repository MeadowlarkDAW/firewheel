/// A font.
#[derive(Debug, Clone, Copy)]
pub enum Font {
    /// The default font.
    ///
    /// This is normally a font configured in a renderer or loaded from the
    /// system.
    Default,

    /// An external font.
    External {
        /// The name of the external font
        name: &'static str,

        /// The bytes of the external font
        bytes: &'static [u8],
    },
}

impl Default for Font {
    fn default() -> Font {
        Font::Default
    }
}

/// The horizontal alignment of text.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum HAlign {
    /// Align text to the left side of the bounds.
    Left,
    /// Align text in the center of the bounds.
    Center,
    /// Align text to the right side of the bounds.
    Right,
}

impl Default for HAlign {
    fn default() -> Self {
        HAlign::Left
    }
}

/// The vertical alignment of text.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum VAlign {
    /// Align text to the top side of the bounds.
    Top,
    /// Align text in the center of the bounds.
    Center,
    /// Align text to the bottom side of the bounds.
    Bottom,
}

impl Default for VAlign {
    fn default() -> Self {
        VAlign::Center
    }
}
