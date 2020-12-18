use crate::Size;

/// The settings of the application
#[derive(Debug, Default)]
pub struct Settings {
    /// The window settings
    pub window: Window,
    /// The antialiasing strategy
    pub antialiasing: Antialiasing,
}

/// The dpi scaling policy of the window
#[derive(Debug, Copy, Clone)]
pub enum ScalePolicy {
    /// Use the system's dpi scale factor
    SystemScaleFactor,
    /// Use the given dpi scale factor (e.g. `1.0` = 96 dpi)
    ScaleFactor(f64),
}

impl Default for ScalePolicy {
    fn default() -> Self {
        ScalePolicy::SystemScaleFactor
    }
}

/// The options for opening a new window
#[derive(Debug)]
pub struct Window {
    pub title: String,

    /// The logical size of the window.
    ///
    /// These dimensions will be scaled by the scaling policy specified in `scale`. Mouse
    /// position will be passed back as logical coordinates.
    pub logical_size: Size,

    /// The dpi scaling policy
    pub scale: ScalePolicy,
}

impl Default for Window {
    fn default() -> Self {
        Window {
            title: String::from("Goldenrod"),
            logical_size: Size::new(600.0, 400.0),
            scale: ScalePolicy::default(),
        }
    }
}

/// An antialiasing strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Antialiasing {
    /// Multisample AA with 2 samples
    MSAAx2,
    /// Multisample AA with 4 samples
    MSAAx4,
    /// Multisample AA with 8 samples
    MSAAx8,
    /// Multisample AA with 16 samples
    MSAAx16,
}

impl Antialiasing {
    /// Returns the amount of samples of the [`Antialiasing`].
    pub fn sample_count(self) -> u32 {
        match self {
            Antialiasing::MSAAx2 => 2,
            Antialiasing::MSAAx4 => 4,
            Antialiasing::MSAAx8 => 8,
            Antialiasing::MSAAx16 => 16,
        }
    }
}

impl Default for Antialiasing {
    fn default() -> Self {
        Antialiasing::MSAAx8
    }
}
