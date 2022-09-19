use fnv::FnvHashMap;
use std::cell::RefCell;
use std::error::Error;
use std::fmt;
use std::rc::Rc;

use crate::layer::{Layer, LayerBuilder, LayerID};
use crate::Size;

pub struct Canvas {
    current_size: Size,
    min_size: Option<Size>,
    max_size: Option<Size>,

    layers: FnvHashMap<LayerID, Rc<RefCell<Layer>>>,
    layers_ordered: Vec<(i32, Vec<Rc<RefCell<Layer>>>)>,

    do_repack_layers: bool,
}

impl Canvas {
    pub fn new(mut intial_size: Size, min_size: Option<Size>, max_size: Option<Size>) -> Self {
        if let Some(min_size) = min_size {
            intial_size = intial_size.max(min_size);

            if let Some(max_size) = max_size {
                assert!(min_size.width() <= max_size.width());
                assert!(min_size.height() <= max_size.height());
            }
        }
        if let Some(max_size) = max_size {
            intial_size = intial_size.min(max_size);
        }

        Self {
            current_size: intial_size,
            min_size,
            max_size,
            layers: FnvHashMap::default(),
            layers_ordered: Vec::new(),
            do_repack_layers: true,
        }
    }

    pub fn add_layer(&mut self, mut layer: Layer) -> Result<(), AddLayerError> {
        let id = layer.id();

        if self.layers.contains_key(&id) {
            return Err(AddLayerError::LayerIDAlreadyExists(layer.id()));
        }

        if let Some(min_size) = layer.min_size {
            if let Some(max_size) = layer.max_size {
                if min_size.width() > max_size.width() || min_size.height() > max_size.height() {
                    return Err(AddLayerError::MinSizeNotLessThanMaxSize { min_size, max_size });
                }
            }

            if let Some(current_size) = &mut layer.current_size {
                *current_size = current_size.max(min_size);
            }
        }
        if let Some(max_size) = layer.max_size {
            if let Some(current_size) = &mut layer.current_size {
                *current_size = current_size.min(max_size);
            }
        }

        let layer = Rc::new(RefCell::new(layer));

        self.layers.insert(id, Rc::clone(&layer));

        let mut existing_z_order_i = None;
        let mut insert_i = 0;
        for (i, (z_order, _)) in self.layers_ordered.iter().enumerate() {
            if id.z_order == *z_order {
                existing_z_order_i = Some(i);
                break;
            } else if id.z_order > *z_order {
                insert_i = i + 1;
            }
        }
        if let Some(i) = existing_z_order_i {
            self.layers_ordered[i].1.push(layer);
        } else {
            self.layers_ordered
                .insert(insert_i, (id.z_order, vec![layer]));
        }

        self.do_repack_layers = true;

        Ok(())
    }

    pub fn remove_layer(&mut self, id: LayerID) {
        if self.layers.remove(&id).is_none() {
            return;
        }

        let mut remove_z_order_i = None;
        for (z_order_i, (z_order, layers)) in self.layers_ordered.iter_mut().enumerate() {
            if id.z_order == *z_order {
                let mut remove_i = None;
                for (i, layer) in layers.iter().enumerate() {
                    if RefCell::borrow(layer).id() == id {
                        remove_i = Some(i);
                        break;
                    }
                }
                if let Some(i) = remove_i {
                    layers.remove(i);

                    if layers.is_empty() {
                        remove_z_order_i = Some(z_order_i);
                    }
                }

                break;
            }
        }
        if let Some(i) = remove_z_order_i {
            self.layers_ordered.remove(i);
        }

        self.do_repack_layers = true;
    }

    fn pack_layers(&mut self) {
        // TODO
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AddLayerError {
    LayerIDAlreadyExists(LayerID),
    MinSizeNotLessThanMaxSize { min_size: Size, max_size: Size },
}

impl Error for AddLayerError {}

impl fmt::Display for AddLayerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LayerIDAlreadyExists(id) => {
                write!(f, "Layer with ID {:?} already exists", id)
            }
            Self::MinSizeNotLessThanMaxSize { min_size, max_size } => {
                write!(
                    f,
                    "Layer has a minimum size {:?} that is not less than its maximum size {:?}",
                    min_size, max_size
                )
            }
        }
    }
}
