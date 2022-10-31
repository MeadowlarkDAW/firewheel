use femtovg::Color;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GradientDirection {
    Horizontal,
    Vertical,
    // TODO: Angle
}

#[derive(Debug, Clone, PartialEq)]
pub enum BgColor {
    Solid(Color),
    LinearGradient {
        direction: GradientDirection,
        /// The gradient stops (maximum of 24 stops).
        /// 
        /// `(percentage in the range [0.0..100.0], Color)`
        stop: Vec<(f32, Color)>,
    },
}