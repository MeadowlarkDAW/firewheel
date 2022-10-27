use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FirewheelError {
    LayerRemoved,
    LayerNotEmpty,
    ParentAnchorRegionNotPartOfLayer,
    ParentAnchorRegionRemoved,
    ContainerRegionRemoved,
    ContainerRegionNotEmpty,
}

impl Error for FirewheelError {}

impl fmt::Display for FirewheelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LayerRemoved => {
                write!(f, "Layer is invalid because it has been removed")
            }
            Self::LayerNotEmpty => {
                write!(f, "Could not remove layer: layer is not empty")
            }
            Self::ParentAnchorRegionNotPartOfLayer => {
                write!(f, "Parent anchor region is invalid because it does not belong to the specified layer")
            }
            Self::ParentAnchorRegionRemoved => {
                write!(
                    f,
                    "Parent anchor region is invalid because it has been removed"
                )
            }
            Self::ContainerRegionRemoved => {
                write!(f, "Container region is invalid because it has been removed")
            }
            Self::ContainerRegionNotEmpty => {
                write!(
                    f,
                    "Could not remove container region: container region is not empty"
                )
            }
        }
    }
}
