use super::allocation;
use crate::Point;

#[derive(Debug)]
pub enum Entry {
    Contiguous {
        allocation: allocation::Allocation,
        rotation_origin: Point<f32>,
        hi_dpi: u32,
    },
    Fragmented {
        size: [f32; 2],
        fragments: Vec<Fragment>,
        rotation_origin: Point<f32>,
        hi_dpi: u32,
    },
}

impl Entry {
    pub fn size(&self) -> [f32; 2] {
        match self {
            Entry::Contiguous { allocation, .. } => allocation.size(),
            Entry::Fragmented { size, .. } => *size,
        }
    }

    pub fn rotation_origin(&self) -> Point<f32> {
        match self {
            Entry::Contiguous {
                rotation_origin, ..
            } => *rotation_origin,
            Entry::Fragmented {
                rotation_origin, ..
            } => *rotation_origin,
        }
    }
}

#[derive(Debug)]
pub struct Fragment {
    pub position: [f32; 2],
    pub allocation: allocation::Allocation,
}
