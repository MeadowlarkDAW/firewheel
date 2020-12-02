use super::allocator;
use crate::{Point, Rect, Size};

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
            Allocation::Partial { region, .. } => region.area.top_left.into(),
            Allocation::Full { .. } => [0.0, 0.0],
        }
    }

    pub fn size(&self) -> [f32; 2] {
        match self {
            Allocation::Partial { region, .. } => region.area.size.into(),
            Allocation::Full { .. } => {
                [super::ATLAS_SIZE as f32, super::ATLAS_SIZE as f32]
            }
        }
    }

    pub fn area(&self) -> Rect {
        match self {
            Allocation::Partial { region, .. } => region.area,
            Allocation::Full { .. } => Rect::new(
                Point::ORIGIN,
                Size::new(super::ATLAS_SIZE as f32, super::ATLAS_SIZE as f32),
            ),
        }
    }

    pub fn layer(&self) -> u32 {
        match self {
            Allocation::Partial { layer, .. } => *layer,
            Allocation::Full { layer, .. } => *layer,
        }
    }
}
