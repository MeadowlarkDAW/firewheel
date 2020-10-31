use super::allocation;
use crate::Point;

#[derive(Debug)]
pub enum Entry {
    Contiguous {
        allocation: allocation::Allocation,
        rotation_origin: Point,
    },
    Fragmented {
        size: (f32, f32),
        fragments: Vec<Fragment>,
        rotation_origin: Point,
    },
}

impl Entry {
    pub fn size(&self) -> (f32, f32) {
        match self {
            Entry::Contiguous { allocation, .. } => allocation.size(),
            Entry::Fragmented { size, .. } => *size,
        }
    }

    pub fn rotation_origin(&self) -> Point {
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
    pub position: (u32, u32),
    pub allocation: allocation::Allocation,
}
