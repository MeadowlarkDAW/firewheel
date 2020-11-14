use super::allocator;
use crate::{Rectangle, Size};

#[derive(Debug)]
pub enum Allocation {
    Partial {
        layer: u32,
        region: allocator::Region,
    },
    Full {
        layer: u32,
    },
}

impl Allocation {
    pub fn position(&self) -> [f32; 2] {
        match self {
            Allocation::Partial { region, .. } => region.area.position().into(),
            Allocation::Full { .. } => [0.0, 0.0],
        }
    }

    pub fn size(&self) -> [f32; 2] {
        match self {
            Allocation::Partial { region, .. } => region.area.size().into(),
            Allocation::Full { .. } => {
                [super::ATLAS_SIZE as f32, super::ATLAS_SIZE as f32]
            }
        }
    }

    pub fn area(&self) -> Rectangle {
        match self {
            Allocation::Partial { region, .. } => region.area,
            Allocation::Full { .. } => Rectangle::with_size(Size {
                width: super::ATLAS_SIZE as u16,
                height: super::ATLAS_SIZE as u16,
            }),
        }
    }

    pub fn layer(&self) -> u32 {
        match self {
            Allocation::Partial { layer, .. } => *layer,
            Allocation::Full { layer, .. } => *layer,
        }
    }
}
