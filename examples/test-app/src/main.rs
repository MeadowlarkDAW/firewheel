use firewheel::event::InputEvent;
use firewheel::vg::{Color, FontId, Paint, Path};
use firewheel::{
    Anchor, AppWindow, BackgroundNode, EventCapturedStatus, PaintRegionInfo, ParentAnchorType,
    PhysicalSize, Point, Rect, RegionInfo, ScaleFactor, Size, WidgetNode, WidgetNodeRequests,
    WidgetNodeType, VG,
};
use glutin::config::{Api, ConfigSurfaceTypes, ConfigTemplateBuilder, GlConfig};
use glutin::context::{
    ContextApi, ContextAttributesBuilder, NotCurrentGlContextSurfaceAccessor, Version,
};
use glutin::display::{Display, DisplayApiPreference};
use glutin::prelude::{GlDisplay, GlSurface};
use glutin::surface::{Surface, SurfaceAttributesBuilder, SwapInterval, WindowSurface};
use raw_window_handle::{
    HasRawDisplayHandle, HasRawWindowHandle, RawDisplayHandle, RawWindowHandle,
};
use std::any::Any;
use std::num::NonZeroU32;
use std::time::Instant;
use winit::{
    event::{Event, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .build();

    // --- Set up winit event loop -----------------------------------------------------

    let event_loop = EventLoop::new();
    let raw_display = event_loop.raw_display_handle();
    let window = WindowBuilder::new()
        .with_title("Firewheel Test App")
        .build(&event_loop)
        .unwrap();
    let raw_window_handle = window.raw_window_handle();

    // --- Set up glutin context -------------------------------------------------------

    let gl_display = unsafe {
        let preference = DisplayApiPreference::GlxThenEgl(Box::new(
            winit::platform::unix::register_xlib_error_hook,
        ));
        Display::new(raw_display, preference).unwrap()
    };

    let gl_config_template = ConfigTemplateBuilder::new()
        .compatible_with_native_window(raw_window_handle)
        .with_surface_type(ConfigSurfaceTypes::WINDOW)
        .build();
    let num_samples = 0;
    let gl_config = unsafe { gl_display.find_configs(gl_config_template) }
        .unwrap()
        .reduce(|accum, config| {
            // Pick a config with the desired number of samples.
            if config.num_samples() == num_samples {
                config
            } else {
                accum
            }
        })
        .unwrap();

    println!("Picked a config with {} samples", gl_config.num_samples());

    let gl_context_attributes = ContextAttributesBuilder::new()
        .with_context_api(ContextApi::OpenGl(None))
        .build(Some(raw_window_handle));

    let mut not_current_gl_context = Some(unsafe {
        gl_display
            .create_context(&gl_config, &gl_context_attributes)
            .expect("failed to create context")
    });

    let gl_surface_attributes = {
        let (width, height): (u32, u32) = window.inner_size().into();
        let raw_window_handle = window.raw_window_handle();
        SurfaceAttributesBuilder::<WindowSurface>::new().build(
            raw_window_handle,
            NonZeroU32::new(width).unwrap(),
            NonZeroU32::new(height).unwrap(),
        )
    };

    let gl_surface = unsafe {
        gl_display
            .create_window_surface(&gl_config, &gl_surface_attributes)
            .unwrap()
    };

    let current_gl_context = not_current_gl_context
        .take()
        .unwrap()
        .make_current(&gl_surface)
        .unwrap();

    // Try setting vsync
    if let Err(res) = gl_surface.set_swap_interval(
        &current_gl_context,
        SwapInterval::Wait(NonZeroU32::new(1).unwrap()),
    ) {
        eprintln!("Error setting vsync: {:?}", res);
    }

    // --- Initialize Firewheel app window ---------------------------------------------

    let mut app_window =
        AppWindow::<MyMsg>::new_from_glutin_display(window.scale_factor().into(), &gl_display);

    let mut window_size = PhysicalSize::new(window.inner_size().width, window.inner_size().height);
    let mut scale_factor = window.scale_factor().into();
    let mut window_logical_size = window_size.to_logical(scale_factor);
    let mut msg_out_queue = Vec::new();

    let main_font_id = app_window
        .vg()
        .add_font("examples/assets/Roboto-Regular.ttf")
        .unwrap();

    let mut test_background_node_ref = app_window.add_background_node(
        window_logical_size,
        0,
        Point::new(0.0, 0.0),
        true,
        Box::new(TestBackgroundNode {}),
    );

    let mut widget_layer_ref = app_window.add_widget_layer(
        window_logical_size,
        1,
        Point::new(0.0, 0.0),
        Point::new(0.0, 0.0),
        true,
    );

    let test_button = TestLabelButton::new(
        main_font_id,
        18.0,
        Color::rgb(235, 235, 235),
        "Hello World!".into(),
        8,
        8,
        0,
        0,
        1.0,
        5.0,
        scale_factor,
        app_window.vg(),
    );
    //let test_button_size = test_button.compute_size(scale_factor, app_window.vg());
    let test_button_size = test_button.estimated_size();
    let mut test_button_ref = app_window.add_widget_node(
        Box::new(test_button),
        &widget_layer_ref,
        RegionInfo {
            size: test_button_size,
            internal_anchor: Anchor::center(),
            parent_anchor: Anchor::center(),
            parent_anchor_type: ParentAnchorType::Layer,
            anchor_offset: Point::new(0.0, 0.0),
        },
        true,
        &mut msg_out_queue,
    );

    // --- Run event loop --------------------------------------------------------------

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() => match event {
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            WindowEvent::Resized(physical_size) => {
                if physical_size.width != 0 && physical_size.height != 0 {
                    gl_surface.resize(
                        &current_gl_context,
                        NonZeroU32::new(physical_size.width).unwrap(),
                        NonZeroU32::new(physical_size.height).unwrap(),
                    );

                    window_size = PhysicalSize::new(physical_size.width, physical_size.height);
                    window_logical_size = window_size.to_logical(scale_factor);

                    app_window
                        .set_background_node_size(
                            &mut test_background_node_ref,
                            window_logical_size,
                        )
                        .unwrap();
                    app_window
                        .set_widget_layer_size(
                            &mut widget_layer_ref,
                            window_logical_size,
                            &mut msg_out_queue,
                        )
                        .unwrap();
                }
            }
            WindowEvent::ScaleFactorChanged {
                scale_factor: window_scale_factor,
                new_inner_size,
            } => {
                if new_inner_size.width != 0 && new_inner_size.height != 0 {
                    gl_surface.resize(
                        &current_gl_context,
                        NonZeroU32::new(new_inner_size.width).unwrap(),
                        NonZeroU32::new(new_inner_size.height).unwrap(),
                    );

                    scale_factor = (*window_scale_factor).into();
                    app_window.set_scale_factor(scale_factor, &mut msg_out_queue);

                    window_size = PhysicalSize::new(new_inner_size.width, new_inner_size.height);
                    window_logical_size = window_size.to_logical(scale_factor);

                    app_window
                        .set_background_node_size(
                            &mut test_background_node_ref,
                            window_logical_size,
                        )
                        .unwrap();
                    app_window
                        .set_widget_layer_size(
                            &mut widget_layer_ref,
                            window_logical_size,
                            &mut msg_out_queue,
                        )
                        .unwrap();
                }
            }
            _ => {}
        },
        Event::RedrawRequested(window_id) if window_id == window.id() => {
            app_window.render(window_size, Color::rgb(30, 30, 30));

            gl_surface.swap_buffers(&current_gl_context).unwrap();
        }
        Event::MainEventsCleared => {
            if app_window.is_dirty() {
                window.request_redraw();
            }
        }
        _ => {}
    });
}

enum MyMsg {}

struct TestBackgroundNode {}

impl BackgroundNode for TestBackgroundNode {
    fn on_user_event(&mut self, event: Box<dyn Any>) -> bool {
        false
    }

    fn paint(&mut self, vg: &mut VG, region: &PaintRegionInfo) {
        const MARGIN: f32 = 4.0;

        let mut path = Path::new();
        path.rounded_rect(
            region.physical_rect.pos.x as f32 + (MARGIN * region.scale_factor.as_f32()).round(),
            region.physical_rect.pos.y as f32 + (MARGIN * region.scale_factor.as_f32()).round(),
            region.physical_rect.size.width as f32
                - (MARGIN * 2.0 * region.scale_factor.as_f32()).round(),
            region.physical_rect.size.height as f32
                - (MARGIN * 2.0 * region.scale_factor.as_f32()).round(),
            18.0 * region.scale_factor.as_f32(),
        );

        let gradient_paint = Paint::linear_gradient(
            region.physical_rect.pos.x as f32,
            region.physical_rect.pos.y as f32,
            region.physical_rect.pos.x as f32,
            region.physical_rect.pos.y as f32 + (19.0 * region.scale_factor.as_f32()),
            Color::rgb(64, 64, 64),
            Color::rgb(50, 50, 50),
        );

        let mut border_paint = Paint::color(Color::rgb(6, 6, 6));
        border_paint.set_line_width((1.0 * region.scale_factor.as_f32()).round());

        vg.fill_path(&mut path, &gradient_paint);
        vg.stroke_path(&mut path, &border_paint);
    }
}

enum ButtonState {
    Idle,
    Hovered,
    Active,
}

struct TestLabelButton {
    label: String,
    padding_lr_pts: u16,
    padding_tb_pts: u16,
    margin_lr_pts: u16,
    margin_tb_pts: u16,

    font_id: FontId,
    font_size_pts: f32,
    font_color: Color,

    border_width_pts: f32,
    border_radius_pts: f32,
    font_rect_size_pts: Size,
    estimated_size_pts: Size,

    state: ButtonState,
}

impl TestLabelButton {
    pub fn new(
        font_id: FontId,
        font_size_pts: f32,
        font_color: Color,
        label: String,
        padding_lr_pts: u16,
        padding_tb_pts: u16,
        margin_lr_pts: u16,
        margin_tb_pts: u16,
        border_width_pts: f32,
        border_radius_pts: f32,
        scale_factor: ScaleFactor,
        vg: &VG,
    ) -> Self {
        let mut new_self = Self {
            label,
            padding_lr_pts,
            padding_tb_pts,
            margin_lr_pts,
            margin_tb_pts,
            font_id,
            font_size_pts,
            font_color,
            border_width_pts,
            border_radius_pts,
            font_rect_size_pts: Size::default(),
            estimated_size_pts: Size::default(),
            state: ButtonState::Idle,
        };

        new_self.compute_size(scale_factor, vg);

        new_self
    }

    pub fn estimated_size(&self) -> Size {
        self.estimated_size_pts
    }

    fn compute_size(&mut self, scale_factor: ScaleFactor, vg: &VG) {
        let mut font_paint = Paint::color(Color::black());
        font_paint.set_font(&[self.font_id]);
        font_paint.set_font_size(self.font_size_pts * scale_factor.0);
        font_paint.set_text_baseline(firewheel::vg::Baseline::Middle);

        let font_metrics = vg.measure_text(0.0, 0.0, &self.label, &font_paint).unwrap();

        self.font_rect_size_pts = Size::new(
            font_metrics.width() / scale_factor.0,
            font_metrics.height() / scale_factor.0,
        );

        let full_width_pts = (self.font_rect_size_pts.width()
            + (f32::from(self.padding_lr_pts + self.margin_lr_pts) * 2.0))
            .ceil();
        let full_height_pts = (self.font_rect_size_pts.height()
            + (f32::from(self.padding_tb_pts + self.margin_tb_pts) * 2.0))
            .ceil();

        self.estimated_size_pts = Size::new(full_width_pts, full_height_pts);
    }
}

impl<MyMsg> WidgetNode<MyMsg> for TestLabelButton {
    fn on_added(&mut self, _msg_out_queue: &mut Vec<MyMsg>) -> WidgetNodeType {
        WidgetNodeType::Painted
    }

    fn on_visibility_hidden(&mut self, _msg_out_queue: &mut Vec<MyMsg>) {
        println!("button hidden");
    }

    fn on_region_changed(&mut self, _assigned_rect: Rect) {
        //println!("region changed");
    }

    fn on_user_event(
        &mut self,
        event: Box<dyn Any>,
        _msg_out_queue: &mut Vec<MyMsg>,
    ) -> Option<WidgetNodeRequests> {
        None
    }

    fn on_input_event(
        &mut self,
        event: &InputEvent,
        msg_out_queue: &mut Vec<MyMsg>,
    ) -> EventCapturedStatus {
        EventCapturedStatus::NotCaptured
    }

    #[allow(unused)]
    fn paint(&mut self, vg: &mut VG, region: &PaintRegionInfo) {
        let mut bg_path = region.spanning_rounded_rect_path(
            self.margin_lr_pts,
            self.margin_tb_pts,
            self.border_width_pts,
            self.border_radius_pts,
        );

        let mut bg_paint = Paint::color(Color::rgb(41, 41, 41));
        let mut bg_stroke_paint = Paint::color(Color::rgb(22, 22, 22));
        bg_stroke_paint
            .set_line_width((self.border_width_pts as f32 * region.scale_factor.0).round());

        vg.fill_path(&mut bg_path, &bg_paint);
        vg.stroke_path(&mut bg_path, &bg_stroke_paint);

        let label_rect_width_px = region.physical_rect.size.width as f32
            - (f32::from(self.margin_lr_pts + self.padding_lr_pts) * region.scale_factor.0 * 2.0)
                .round()
                .max(0.0);
        let label_rect_height_px = region.physical_rect.size.height as f32
            - (f32::from(self.margin_tb_pts + self.padding_tb_pts) * region.scale_factor.0 * 2.0)
                .round()
                .max(0.0);

        if label_rect_width_px != 0.0 && label_rect_height_px != 0.0 {
            let label_rect_x_px = region.physical_rect.pos.x as f32
                + (f32::from(self.margin_lr_pts + self.padding_lr_pts) * region.scale_factor.0);
            let label_rect_y_px = region.physical_rect.pos.y as f32
                + (f32::from(self.margin_tb_pts + self.padding_tb_pts) * region.scale_factor.0);
            let label_rect_y_center_px = (region.physical_rect.pos.y as f32
                + (f32::from(self.margin_tb_pts + self.padding_tb_pts) * region.scale_factor.0)
                + (self.font_rect_size_pts.height() * region.scale_factor.0 / 2.0))
                .round()
                - 0.5;

            vg.scissor(
                label_rect_x_px,
                label_rect_y_px,
                label_rect_width_px,
                label_rect_height_px,
            );

            let mut font_paint = Paint::color(self.font_color);
            font_paint.set_font(&[self.font_id]);
            font_paint.set_font_size(self.font_size_pts * region.scale_factor.0);
            font_paint.set_text_baseline(firewheel::vg::Baseline::Middle);

            vg.fill_text(
                label_rect_x_px,
                label_rect_y_center_px,
                &self.label,
                &font_paint,
            );

            vg.reset_scissor();
        }
    }
}
