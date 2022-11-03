use femtovg::{Color, ImageFlags, ImageId, PixelFormat};
//use glow::{HasContext, NativeFramebuffer, NativeTexture};
use std::ffi::c_void;

use crate::{layer::StrongLayerEntry, size::PhysicalSize, AppWindow, ScaleFactor};

mod background_layer_renderer;
mod widget_layer_renderer;
pub(crate) use background_layer_renderer::BackgroundLayerRenderer;
pub(crate) use widget_layer_renderer::WidgetLayerRenderer;

// TODO: Pack multiple layers into a single texture instead of having one
// texture per layer.

pub(crate) struct Renderer {
    pub vg: femtovg::Canvas<femtovg::renderer::OpenGl>,
    //glow_context: glow::Context,
    window_size: PhysicalSize,
    scale_factor: ScaleFactor,
}

impl Renderer {
    #[cfg(all(feature = "glutin", not(target_arch = "wasm32")))]
    pub fn new_from_glutin_display(display: &glutin::display::Display) -> Self {
        use glutin::display::GlDisplay;

        unsafe {
            Self::new_from_function(|symbol| {
                let symbol = std::ffi::CString::new(symbol).unwrap();
                display.get_proc_address(symbol.as_c_str()).cast()
            })
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub unsafe fn new_from_function<F>(mut load_fn: F) -> Self
    where
        F: FnMut(&str) -> *const c_void,
    {
        let vg_renderer = femtovg::renderer::OpenGl::new_from_function(&mut load_fn).unwrap();

        //log::info!("OpenGL renderer is ES: {}", vg_renderer.is_opengles());
        println!("OpenGL renderer is ES: {}", vg_renderer.is_opengles());

        let vg = femtovg::Canvas::new(vg_renderer).unwrap();

        //let glow_context = glow::Context::from_loader_function(load_fn);

        //println!("{:?}", glow_context.version());

        Self {
            vg,
            //glow_context,
            window_size: PhysicalSize::default(),
            scale_factor: ScaleFactor(0.0),
        }
    }

    pub fn render<A: Clone + Send + Sync + 'static>(
        &mut self,
        app_window: &mut AppWindow<A>,
        window_size: PhysicalSize,
        scale_factor: ScaleFactor,
        clear_color: Color,
    ) {
        for mut layer_renderer in app_window.widget_layer_renderers_to_clean_up.drain(..) {
            layer_renderer.clean_up(&mut self.vg);
        }
        for mut layer_renderer in app_window.background_layer_renderers_to_clean_up.drain(..) {
            layer_renderer.clean_up(&mut self.vg);
        }

        /*
        unsafe {
            self.glow_context.bind_framebuffer(glow::FRAMEBUFFER, None);

            self.glow_context.clear_color(
                clear_color.r,
                clear_color.g,
                clear_color.b,
                clear_color.a,
            );
            self.glow_context.clear(glow::COLOR_BUFFER_BIT);
        }
        */

        self.vg.set_render_target(femtovg::RenderTarget::Screen);
        if self.window_size != window_size || self.scale_factor != scale_factor {
            self.window_size = window_size;
            self.scale_factor = scale_factor;

            self.vg.set_size(window_size.width, window_size.height, 1.0);
        }

        self.vg
            .clear_rect(0, 0, window_size.width, window_size.height, clear_color);

        for (_z_order, layer_entries) in app_window.layers_ordered.iter_mut() {
            for layer_entry in layer_entries.iter_mut() {
                match layer_entry {
                    StrongLayerEntry::Widget(layer_entry) => {
                        let mut layer = layer_entry.borrow_mut();
                        if layer.is_visible() {
                            let mut layer_renderer = layer.renderer.take().unwrap();

                            layer_renderer.render(&mut *layer, &mut self.vg, scale_factor);

                            layer.renderer = Some(layer_renderer);
                        }
                    }
                    StrongLayerEntry::Background(layer_entry) => {
                        let mut layer = layer_entry.borrow_mut();
                        if layer.is_visible() {
                            let mut layer_renderer = layer.renderer.take().unwrap();

                            layer_renderer.render(&mut *layer, &mut self.vg, scale_factor);

                            layer.renderer = Some(layer_renderer);
                        }
                    }
                }
            }
        }

        self.vg.flush();

        /*
        unsafe {
            self.glow_context.bind_framebuffer(glow::FRAMEBUFFER, None);
            self.glow_context
                .bind_framebuffer(glow::READ_FRAMEBUFFER, None);
            self.glow_context
                .bind_framebuffer(glow::DRAW_FRAMEBUFFER, None);
        }
        */
    }

    pub fn free<A: Clone + Send + Sync + 'static>(&mut self, app_window: &mut AppWindow<A>) {
        for mut layer_renderer in app_window.widget_layer_renderers_to_clean_up.drain(..) {
            layer_renderer.clean_up(&mut self.vg);
        }
        for mut layer_renderer in app_window.background_layer_renderers_to_clean_up.drain(..) {
            layer_renderer.clean_up(&mut self.vg);
        }
    }
}

struct TextureState {
    texture_id: ImageId,
    physical_size: PhysicalSize,
    freed: bool,
}

impl TextureState {
    fn new(
        physical_size: PhysicalSize,
        vg: &mut femtovg::Canvas<femtovg::renderer::OpenGl>,
    ) -> Self {
        let texture_id = vg
            .create_image_empty(
                physical_size.width as usize,
                physical_size.height as usize,
                PixelFormat::Rgba8,
                ImageFlags::NEAREST,
            )
            .unwrap();

        Self {
            texture_id,
            physical_size,
            freed: false,
        }
    }

    fn resize(
        &mut self,
        physical_size: PhysicalSize,
        vg: &mut femtovg::Canvas<femtovg::renderer::OpenGl>,
    ) {
        if !self.freed {
            vg.delete_image(self.texture_id);

            self.texture_id = vg
                .create_image_empty(
                    physical_size.width as usize,
                    physical_size.height as usize,
                    PixelFormat::Rgba8,
                    ImageFlags::NEAREST,
                )
                .unwrap();

            self.physical_size = physical_size;
        }
    }

    fn free(&mut self, vg: &mut femtovg::Canvas<femtovg::renderer::OpenGl>) {
        if !self.freed {
            vg.delete_image(self.texture_id);

            self.freed = true;
        }
    }
}

/*
struct TextureState {
    native_framebuffer: NativeFramebuffer,
    native_texture: NativeTexture,
    texture_id: ImageId,
    physical_size: PhysicalSize,
    freed: bool,
}

impl TextureState {
    fn new(
        physical_size: PhysicalSize,
        vg: &mut femtovg::Canvas<femtovg::renderer::OpenGl>,
        glow_context: &mut glow::Context,
    ) -> Self {
        let native_framebuffer = unsafe { glow_context.create_framebuffer().unwrap() };

        let native_texture =
            unsafe { create_native_texture(glow_context, physical_size, native_framebuffer) };

        let texture_id = vg
            .create_image_from_native_texture(
                native_texture,
                ImageInfo::new(
                    ImageFlags::NEAREST,
                    physical_size.width as usize,
                    physical_size.height as usize,
                    PixelFormat::Rgb8,
                ),
            )
            .unwrap();

        Self {
            native_framebuffer,
            native_texture,
            texture_id,
            physical_size,
            freed: false,
        }
    }

    fn resize(
        &mut self,
        physical_size: PhysicalSize,
        vg: &mut femtovg::Canvas<femtovg::renderer::OpenGl>,
        glow_context: &mut glow::Context,
    ) {
        self.physical_size = physical_size;

        // Free the old texture
        vg.delete_image(self.texture_id);
        unsafe {
            glow_context.delete_texture(self.native_texture);
        }

        self.native_texture =
            unsafe { create_native_texture(glow_context, physical_size, self.native_framebuffer) };

        self.texture_id = vg
            .create_image_from_native_texture(
                self.native_texture,
                ImageInfo::new(
                    ImageFlags::NEAREST,
                    physical_size.width as usize,
                    physical_size.height as usize,
                    PixelFormat::Rgb8,
                ),
            )
            .unwrap();
    }

    fn free(
        &mut self,
        vg: &mut femtovg::Canvas<femtovg::renderer::OpenGl>,
        glow_context: &mut glow::Context,
    ) {
        if !self.freed {
            vg.delete_image(self.texture_id);
            unsafe {
                glow_context.delete_texture(self.native_texture);
                glow_context.delete_framebuffer(self.native_framebuffer);
            }

            self.freed = true;
        }
    }
}

unsafe fn create_native_texture(
    glow_context: &mut glow::Context,
    physical_size: PhysicalSize,
    native_framebuffer: NativeFramebuffer,
) -> NativeTexture {
    glow_context.bind_framebuffer(glow::FRAMEBUFFER, Some(native_framebuffer));

    let native_texture = glow_context.create_texture().unwrap();

    glow_context.active_texture(glow::TEXTURE0);
    glow_context.bind_texture(glow::TEXTURE_2D, Some(native_texture));
    glow_context.tex_image_2d(
        glow::TEXTURE_2D,
        0,
        glow::RGBA as i32,
        physical_size.width as i32,
        physical_size.height as i32,
        0,
        glow::RGBA,
        glow::UNSIGNED_BYTE,
        None,
    );
    glow_context.tex_parameter_i32(
        glow::TEXTURE_2D,
        glow::TEXTURE_MIN_FILTER,
        glow::NEAREST as i32,
    );
    glow_context.tex_parameter_i32(
        glow::TEXTURE_2D,
        glow::TEXTURE_MAG_FILTER,
        glow::NEAREST as i32,
    );
    glow_context.framebuffer_texture_2d(
        glow::FRAMEBUFFER,
        glow::COLOR_ATTACHMENT0,
        glow::TEXTURE_2D,
        Some(native_texture),
        0,
    );

    glow_context.bind_framebuffer(glow::FRAMEBUFFER, None);

    native_texture
}
*/
