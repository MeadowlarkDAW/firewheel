use crate::size::{Point, Size};
use std::cmp::Ordering;
use std::hash::Hash;

/// The unique identifier for a layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LayerID {
    /// The ID of this layer. Layers in the same `z_order` cannot
    /// have the same ID.
    pub id: u64,
    /// The z-order of this layer.
    pub z_order: i32,
}

impl PartialOrd for LayerID {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LayerID {
    fn cmp(&self, other: &Self) -> Ordering {
        self.z_order.cmp(&other.z_order)
    }
}

pub struct LayerBuilder {
    layer: Layer,
}

impl LayerBuilder {
    pub fn new(id: LayerID) -> Self {
        Self {
            layer: Layer {
                id,
                current_size: None,
                min_size: None,
                max_size: None,
                visible: true,
                dirty: true,
            },
        }
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

pub struct Layer {
    id: LayerID,

    pub(crate) current_size: Option<Size>,
    pub(crate) min_size: Option<Size>,
    pub(crate) max_size: Option<Size>,

    pub(crate) visible: bool,
    pub(crate) dirty: bool,
}

impl Layer {
    pub fn id(&self) -> LayerID {
        self.id
    }
}
