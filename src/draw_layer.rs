use crate::size::{Point, RegionRect, Size};

pub struct DrawLayerBuilder {
    layer: DrawLayer,
}

impl DrawLayerBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_min_size(mut self, min_size: Size) -> Self {
        self.layer.min_size = Some(min_size);
        self
    }

    pub fn with_max_size(mut self, max_size: Size) -> Self {
        self.layer.max_size = Some(max_size);
        self
    }

    pub fn with_fixed_size(mut self, size: Size) -> Self {
        self.layer.min_size = Some(size);
        self.layer.max_size = Some(size);
        self
    }

    pub fn visible(mut self, visible: bool) -> Self {
        self.layer.visible = visible;
        self
    }
}

impl Default for DrawLayerBuilder {
    fn default() -> Self {
        Self {
            layer: DrawLayer {
                current_size: None,
                min_size: None,
                max_size: None,
                visible: true,
                dirty: false,
            },
        }
    }
}

pub struct DrawLayer {
    current_size: Option<Size>,
    min_size: Option<Size>,
    max_size: Option<Size>,

    visible: bool,
    dirty: bool,
}

impl DrawLayer {}