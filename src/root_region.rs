use crate::size::{Point, RegionRect, Size};

pub struct RootRegionBuilder {
    region: RootRegion,
}

impl RootRegionBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_min_size(mut self, min_size: Size) -> Self {
        self.region.min_size = Some(min_size);
        self
    }

    pub fn with_max_size(mut self, max_size: Size) -> Self {
        self.region.max_size = Some(max_size);
        self
    }

    pub fn with_fixed_size(mut self, size: Size) -> Self {
        self.region.min_size = Some(size);
        self.region.max_size = Some(size);
        self
    }

    pub fn visible(mut self, visible: bool) -> Self {
        self.region.visible = visible;
        self
    }
}

impl Default for RootRegionBuilder {
    fn default() -> Self {
        Self {
            region: RootRegion {
                current_size: None,
                min_size: None,
                max_size: None,
                visible: true,
                dirty: false,
            },
        }
    }
}

pub struct RootRegion {
    current_size: Option<Size>,
    min_size: Option<Size>,
    max_size: Option<Size>,

    visible: bool,
    dirty: bool,
}

impl RootRegion {}
