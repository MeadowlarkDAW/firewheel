use crate::Size;

/// The settings of the application
pub struct Settings {
    /// The window settings
    pub window: Window,
}

/// The dpi scaling policy of the window
#[derive(Debug, Copy, Clone)]
pub enum ScalePolicy {
    /// Use the system's dpi scale factor
    SystemScaleFactor,
    /// Use the given dpi scale factor (e.g. `1.0` = 96 dpi)
    ScaleFactor(f64),
}

/// The options for opening a new window
#[derive(Debug)]
pub struct Window {
    pub title: String,

    /// The logical size of the window.
    ///
    /// These dimensions will be scaled by the scaling policy specified in `scale`. Mouse
    /// position will be passed back as logical coordinates.
    pub logical_size: Size<u16>,

    /// The dpi scaling policy
    pub scale: ScalePolicy,
}
