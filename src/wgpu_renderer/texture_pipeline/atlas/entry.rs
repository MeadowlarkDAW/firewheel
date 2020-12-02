use super::allocation;
use crate::Point;

#[derive(Debug)]
pub enum Entry {
    Contiguous {
        allocation: allocation::Allocation,
        center: Point,
        hi_dpi: u32,
    },
    Fragmented {
        size: [f32; 2],
        fragments: Vec<Fragment>,
        center: Point,
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

    pub fn center(&self) -> Point {
        match self {
            Entry::Contiguous { center, .. } => *center,
            Entry::Fragmented { center, .. } => *center,
        }
    }
}

#[derive(Debug)]
pub struct Fragment {
    pub position: [f32; 2],
    pub allocation: allocation::Allocation,
}
