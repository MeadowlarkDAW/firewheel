use crate::{font, Font, Rectangle};
use std::{cell::RefCell, collections::HashMap};
use wgpu_glyph::{ab_glyph, GlyphBrush, Layout, Section, Text};

pub struct Pipeline {
    glyph_brush: RefCell<GlyphBrush<()>>,
    font_map: RefCell<HashMap<String, wgpu_glyph::FontId>>,
}

impl Pipeline {
    pub fn new(
        device: &wgpu::Device,
        texture_format: wgpu::TextureFormat,
        default_font: Option<&[u8]>,
    ) -> Self {
        let default_font = default_font.map(|slice| slice.to_vec());

        #[cfg(feature = "font-fallback")]
        let default_font =
            default_font.unwrap_or_else(|| font::FALLBACK.to_vec());
        #[cfg(feature = "font-fallback")]
        let font = ab_glyph::FontArc::try_from_vec(default_font)
            .unwrap_or_else(|_| {
                /* log::warn!(
                    "System font failed to load. Falling back to \
                    embedded font..."
                ); */
                ab_glyph::FontArc::try_from_slice(font::FALLBACK)
                    .expect("Failed to load fallback font")
            });

        #[cfg(not(feature = "font-fallback"))]
        let font =
            ab_glyph::FontArc::try_from_vec(default_font.unwrap()).unwrap();

        let glyph_brush =
            wgpu_glyph::GlyphBrushBuilder::using_font(font.clone())
                .initial_cache_size((2048, 2048))
                .draw_cache_multithread(false) // TODO: Expose as a configuration flag
                .build(device, texture_format);

        Pipeline {
            glyph_brush: RefCell::new(glyph_brush),
            font_map: RefCell::new(HashMap::new()),
        }
    }

    pub fn queue(&mut self, section: wgpu_glyph::Section<'_>) {
        self.glyph_brush.borrow_mut().queue(section);
    }

    pub fn render(
        &mut self,
        device: &wgpu::Device,
        staging_belt: &mut wgpu::util::StagingBelt,
        encoder: &mut wgpu::CommandEncoder,
        projection_scale: [f32; 2],
        bounds: Rectangle,
        target: &wgpu::TextureView,
    ) {
        let section = Section::new()
            .with_layout(Layout::default_single_line())
            .add_text(
                Text::new("Hello World!")
                    .with_color([1.0, 1.0, 1.0, 1.0])
                    .with_scale(20.0),
            );

        self.queue(section);

        self.glyph_brush
            .borrow_mut()
            .draw_queued(
                device,
                staging_belt,
                encoder,
                target,
                bounds.width.round() as u32,
                bounds.height.round() as u32,
            )
            .expect("Error rendering text");
    }

    pub fn get_font_id(&self, font: Font) -> wgpu_glyph::FontId {
        match font {
            Font::Default => wgpu_glyph::FontId(0),
            Font::External { name, bytes } => {
                if let Some(font_id) = self.font_map.borrow().get(name) {
                    return *font_id;
                }

                let font = ab_glyph::FontArc::try_from_slice(bytes)
                    .expect("Error loading font");

                let font_id = self.glyph_brush.borrow_mut().add_font(font);

                let _ = self
                    .font_map
                    .borrow_mut()
                    .insert(String::from(name), font_id);

                font_id
            }
        }
    }
}
