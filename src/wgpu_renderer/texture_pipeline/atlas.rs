use crate::{texture, Point};
use image::{ImageBuffer, ImageError};
use std::collections::HashMap;
use std::error::Error;
use std::fmt::{self, Debug};

mod allocation;
mod allocator;
mod entry;
mod layer;

pub use allocation::Allocation;
pub use entry::Entry;
pub use layer::Layer;

use allocator::Allocator;

pub const ATLAS_SIZE: u32 = 2048;

#[derive(Debug)]
pub enum AtlasError {
    ImageError(ImageError, String),
    PixelBufferTooSmall(u32, u32),
    Unkown,
}

impl fmt::Display for AtlasError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AtlasError::ImageError(ref e, path) => {
                write!(f, "Image Error: {}, {}", e, path)
            }
            AtlasError::PixelBufferTooSmall(width, height) => {
                write!(f, "The pixel buffer is smaller than the given size: width: {}, height: {}", width, height)
            }
            AtlasError::Unkown => {
                write!(f, "Unkown error")
            }
        }
    }
}

impl Error for AtlasError {}

pub struct Atlas {
    texture: wgpu::Texture,
    texture_view: wgpu::TextureView,
    layers: Vec<Layer>,
    atlas_map: HashMap<u64, Entry>,
    did_load_once: bool,
}

impl Atlas {
    pub fn new(device: &wgpu::Device) -> Self {
        let extent = wgpu::Extent3d {
            width: ATLAS_SIZE,
            height: ATLAS_SIZE,
            depth: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("goldenrod::atlas texture atlas"),
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            usage: wgpu::TextureUsage::COPY_DST
                | wgpu::TextureUsage::COPY_SRC
                | wgpu::TextureUsage::SAMPLED,
        });

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        });

        Atlas {
            texture,
            texture_view,
            layers: vec![Layer::Empty],
            atlas_map: HashMap::new(),
            did_load_once: false,
        }
    }

    pub fn replace_texture_atlas(
        &mut self,
        device: &wgpu::Device,
        textures: &[(u64, &texture::Texture)],
        encoder: &mut wgpu::CommandEncoder,
        hi_dpi: bool,
    ) -> Result<(), AtlasError> {
        let mut collected_textures: Vec<(
            u64,
            ImageBuffer<image::Bgra<u8>, Vec<u8>>,
            bool,
            Point<f32>,
        )> = Vec::with_capacity(textures.len());

        for (id, texture) in textures {
            match texture.load_bgra(hi_dpi) {
                Ok((data, is_hi_dpi, rotation_origin)) => {
                    collected_textures.push((
                        *id,
                        data,
                        is_hi_dpi,
                        rotation_origin,
                    ));
                }
                Err(e) => match e {
                    texture::TextureError::ImageError(e, path) => {
                        return Err(AtlasError::ImageError(e, path));
                    }
                    texture::TextureError::PixelBufferTooSmall(
                        width,
                        height,
                    ) => {
                        return Err(AtlasError::PixelBufferTooSmall(
                            width, height,
                        ));
                    }
                },
            }
        }

        // Clear old entries
        self.clear(device);

        self.atlas_map.reserve(collected_textures.len());

        for (id, data, is_hi_dpi, rotation_origin) in collected_textures {
            if let Some(entry) = self.add_new_entry(
                data.width(),
                data.height(),
                data.to_vec().as_slice(),
                is_hi_dpi,
                rotation_origin,
                device,
                encoder,
            ) {
                let _ = self.atlas_map.insert(id, entry);
            } else {
                // Ran out of memory (?)
                return Err(AtlasError::Unkown);
            }
        }

        Ok(())
    }

    pub fn view(&self) -> &wgpu::TextureView {
        &self.texture_view
    }

    pub fn layer_count(&self) -> usize {
        self.layers.len()
    }

    fn clear(&mut self, device: &wgpu::Device) {
        // Don't clear if this is the first time loading textures.
        if self.did_load_once {
            self.layers = vec![Layer::Empty];
            self.atlas_map = HashMap::new();

            // TODO: Clear wgpu texture buffers (?)

            self.texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("goldenrod::atlas texture atlas"),
                size: wgpu::Extent3d {
                    width: ATLAS_SIZE,
                    height: ATLAS_SIZE,
                    depth: self.layers.len() as u32,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                usage: wgpu::TextureUsage::COPY_DST
                    | wgpu::TextureUsage::COPY_SRC
                    | wgpu::TextureUsage::SAMPLED,
            });

            self.texture_view =
                self.texture.create_view(&wgpu::TextureViewDescriptor {
                    dimension: Some(wgpu::TextureViewDimension::D2Array),
                    ..Default::default()
                });
        } else {
            self.did_load_once = true;
        }
    }

    fn add_new_entry(
        &mut self,
        width: u32,
        height: u32,
        data: &[u8],
        hi_dpi: bool,
        rotation_origin: Point<f32>,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
    ) -> Option<Entry> {
        use wgpu::util::DeviceExt;

        let entry = {
            let current_size = self.layers.len();
            let entry =
                self.allocate(width, height, hi_dpi, rotation_origin)?;

            // We grow the internal texture after allocating if necessary
            let new_layers = self.layers.len() - current_size;
            self.grow(new_layers, device, encoder);

            entry
        };

        //log::info!("Allocated atlas entry: {:?}", entry);

        // It is a webgpu requirement that:
        //   BufferCopyView.layout.bytes_per_row % wgpu::COPY_BYTES_PER_ROW_ALIGNMENT == 0
        // So we calculate padded_width by rounding width up to the next
        // multiple of wgpu::COPY_BYTES_PER_ROW_ALIGNMENT.
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let padding = (align - (4 * width) % align) % align;
        let padded_width = (4 * width + padding) as usize;
        let padded_data_size = padded_width * height as usize;

        let mut padded_data = vec![0; padded_data_size];

        for row in 0 as usize..height as usize {
            let offset = row * padded_width;

            padded_data[offset..offset + 4 * width as usize].copy_from_slice(
                &data[row * 4 * width as usize..(row + 1) * 4 * width as usize],
            )
        }

        let buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("goldenrod::atlas texture staging buffer"),
                contents: &padded_data,
                usage: wgpu::BufferUsage::COPY_SRC,
            });

        match &entry {
            Entry::Contiguous { allocation, .. } => {
                self.upload_allocation(
                    &buffer,
                    width,
                    height,
                    padding,
                    0,
                    &allocation,
                    encoder,
                );
            }
            Entry::Fragmented { fragments, .. } => {
                for fragment in fragments {
                    let [x, y] = fragment.position;
                    let offset = (y as u32 * padded_width as u32 + 4 * x as u32)
                        as usize;

                    self.upload_allocation(
                        &buffer,
                        width,
                        height,
                        padding,
                        offset,
                        &fragment.allocation,
                        encoder,
                    );
                }
            }
        }

        //log::info!("Current atlas: {:?}", self);

        Some(entry)
    }

    #[inline]
    pub fn get_entry(&self, texture_id_hash: u64) -> Option<&Entry> {
        self.atlas_map.get(&texture_id_hash)
    }

    /*
    pub fn remove(&mut self, entry: &Entry) {
        //log::info!("Removing atlas entry: {:?}", entry);

        match entry {
            Entry::Contiguous(allocation) => {
                self.deallocate(allocation);
            }
            Entry::Fragmented { fragments, .. } => {
                for fragment in fragments {
                    self.deallocate(&fragment.allocation);
                }
            }
        }
    }
    */

    fn allocate(
        &mut self,
        width: u32,
        height: u32,
        hi_dpi: bool,
        rotation_origin: Point<f32>,
    ) -> Option<Entry> {
        let hi_dpi_u32: u32 = if hi_dpi { 1 } else { 0 };

        // Allocate one layer if texture fits perfectly
        if width == ATLAS_SIZE && height == ATLAS_SIZE {
            let mut empty_layers = self
                .layers
                .iter_mut()
                .enumerate()
                .filter(|(_, layer)| layer.is_empty());

            if let Some((i, layer)) = empty_layers.next() {
                *layer = Layer::Full;

                return Some(Entry::Contiguous {
                    allocation: Allocation::Full { layer: i as u32 },
                    rotation_origin,
                    hi_dpi: hi_dpi_u32,
                });
            }

            self.layers.push(Layer::Full);

            return Some(Entry::Contiguous {
                allocation: Allocation::Full {
                    layer: self.layers.len() as u32 - 1,
                },
                rotation_origin,
                hi_dpi: hi_dpi_u32,
            });
        }

        // Split big textures across multiple layers
        if width > ATLAS_SIZE || height > ATLAS_SIZE {
            let mut fragments = Vec::new();
            let mut y = 0;

            while y < height {
                let height = std::cmp::min(height - y, ATLAS_SIZE);
                let mut x = 0;

                while x < width {
                    let width = std::cmp::min(width - x, ATLAS_SIZE);

                    let allocation =
                        self.allocate(width, height, hi_dpi, rotation_origin)?;

                    if let Entry::Contiguous { allocation, .. } = allocation {
                        fragments.push(entry::Fragment {
                            position: [x as f32, y as f32],
                            allocation,
                        });
                    }

                    x += width;
                }

                y += height;
            }

            return Some(Entry::Fragmented {
                size: [width as f32, height as f32],
                fragments,
                rotation_origin,
                hi_dpi: hi_dpi_u32,
            });
        }

        // Try allocating on an existing layer
        for (i, layer) in self.layers.iter_mut().enumerate() {
            match layer {
                Layer::Empty => {
                    let mut allocator = Allocator::new(ATLAS_SIZE);

                    if let Some(region) = allocator.allocate(width, height) {
                        *layer = Layer::Busy(allocator);

                        return Some(Entry::Contiguous {
                            allocation: Allocation::Partial {
                                region,
                                layer: i as u32,
                            },
                            rotation_origin,
                            hi_dpi: hi_dpi_u32,
                        });
                    }
                }
                Layer::Busy(allocator) => {
                    if let Some(region) = allocator.allocate(width, height) {
                        return Some(Entry::Contiguous {
                            allocation: Allocation::Partial {
                                region,
                                layer: i as u32,
                            },
                            rotation_origin,
                            hi_dpi: hi_dpi_u32,
                        });
                    }
                }
                _ => {}
            }
        }

        // Create new layer with atlas allocator
        let mut allocator = Allocator::new(ATLAS_SIZE);

        if let Some(region) = allocator.allocate(width, height) {
            self.layers.push(Layer::Busy(allocator));

            return Some(Entry::Contiguous {
                allocation: Allocation::Partial {
                    region,
                    layer: self.layers.len() as u32 - 1,
                },
                rotation_origin,
                hi_dpi: hi_dpi_u32,
            });
        }

        // We ran out of memory (?)
        None
    }

    /*
    fn deallocate(&mut self, allocation: &Allocation) {
        //log::info!("Deallocating atlas: {:?}", allocation);

        match allocation {
            Allocation::Full { layer } => {
                self.layers[*layer] = Layer::Empty;
            }
            Allocation::Partial { layer, region } => {
                let layer = &mut self.layers[*layer];

                if let Layer::Busy(allocator) = layer {
                    allocator.deallocate(region);

                    if allocator.is_empty() {
                        *layer = Layer::Empty;
                    }
                }
            }
        }
    }
    */

    fn upload_allocation(
        &mut self,
        buffer: &wgpu::Buffer,
        image_width: u32,
        image_height: u32,
        padding: u32,
        offset: usize,
        allocation: &Allocation,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let [x, y] = allocation.position();
        let [width, height] = allocation.size();
        let layer = allocation.layer();

        let extent = wgpu::Extent3d {
            width: width as u32,
            height: height as u32,
            depth: 1,
        };

        encoder.copy_buffer_to_texture(
            wgpu::BufferCopyView {
                buffer,
                layout: wgpu::TextureDataLayout {
                    offset: offset as u64,
                    bytes_per_row: 4 * image_width + padding,
                    rows_per_image: image_height,
                },
            },
            wgpu::TextureCopyView {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: x as u32,
                    y: y as u32,
                    z: layer as u32,
                },
            },
            extent,
        );
    }

    fn grow(
        &mut self,
        amount: usize,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        if amount == 0 {
            return;
        }

        let new_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("goldenrod::atlas texture atlas"),
            size: wgpu::Extent3d {
                width: ATLAS_SIZE,
                height: ATLAS_SIZE,
                depth: self.layers.len() as u32,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            usage: wgpu::TextureUsage::COPY_DST
                | wgpu::TextureUsage::COPY_SRC
                | wgpu::TextureUsage::SAMPLED,
        });

        let amount_to_copy = self.layers.len() - amount;

        for (i, layer) in
            self.layers.iter_mut().take(amount_to_copy).enumerate()
        {
            if layer.is_empty() {
                continue;
            }

            encoder.copy_texture_to_texture(
                wgpu::TextureCopyView {
                    texture: &self.texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: 0,
                        y: 0,
                        z: i as u32,
                    },
                },
                wgpu::TextureCopyView {
                    texture: &new_texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: 0,
                        y: 0,
                        z: i as u32,
                    },
                },
                wgpu::Extent3d {
                    width: ATLAS_SIZE,
                    height: ATLAS_SIZE,
                    depth: 1,
                },
            );
        }

        self.texture = new_texture;
        self.texture_view =
            self.texture.create_view(&wgpu::TextureViewDescriptor {
                dimension: Some(wgpu::TextureViewDimension::D2Array),
                ..Default::default()
            });
    }
}
