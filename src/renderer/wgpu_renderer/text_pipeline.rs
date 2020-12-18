use crate::{font, Color, Font, HAlign, Point, Rect, Size, VAlign};
use std::{cell::RefCell, collections::HashMap};
use wgpu_glyph::{
    ab_glyph, BuiltInLineBreaker, GlyphBrush, HorizontalAlign, Layout, Section,
    Text, VerticalAlign,
};

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

    pub fn add_single_line(
        &mut self,
        text: &str,
        font_color: Color,
        font_size: f32,
        font_family: font::Font,
        position: Point,
        scissor_rect: Option<Size>,
        h_align: HAlign,
        v_align: VAlign,
    ) {
        let font_id = self.get_font_id(font_family);
        let position = (f32::from(position.x), f32::from(position.y));

        let h_align = match h_align {
            HAlign::Center => HorizontalAlign::Center,
            HAlign::Left => HorizontalAlign::Left,
            HAlign::Right => HorizontalAlign::Right,
        };
        let v_align = match v_align {
            VAlign::Center => VerticalAlign::Center,
            VAlign::Bottom => VerticalAlign::Bottom,
            VAlign::Top => VerticalAlign::Top,
        };

        let section = if let Some(scissor_rect) = scissor_rect {
            let bounds = (scissor_rect.width(), scissor_rect.height());

            Section::new()
                .with_layout(Layout::SingleLine {
                    line_breaker: BuiltInLineBreaker::default(),
                    h_align,
                    v_align,
                })
                .add_text(
                    Text::new(text)
                        .with_color(font_color)
                        .with_scale(font_size)
                        .with_font_id(font_id),
                )
                .with_screen_position(position)
                .with_bounds(bounds)
        } else {
            Section::new()
                .with_layout(Layout::SingleLine {
                    line_breaker: BuiltInLineBreaker::default(),
                    h_align,
                    v_align,
                })
                .add_text(
                    Text::new(text)
                        .with_color(font_color)
                        .with_scale(font_size)
                        .with_font_id(font_id),
                )
                .with_screen_position(position)
        };

        self.glyph_brush.borrow_mut().queue(section);
    }

    pub fn render(
        &mut self,
        device: &wgpu::Device,
        staging_belt: &mut wgpu::util::StagingBelt,
        encoder: &mut wgpu::CommandEncoder,
        bounds: Rect,
        target: &wgpu::TextureView,
    ) {
        self.glyph_brush
            .borrow_mut()
            .draw_queued(
                device,
                staging_belt,
                encoder,
                target,
                bounds.size.width() as u32,
                bounds.size.height() as u32,
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
