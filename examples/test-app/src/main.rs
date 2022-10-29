use firewheel::event::InputEvent;
use firewheel::vg::Color;
use firewheel::{
    AppWindow, BackgroundNode, EventCapturedStatus, PaintRegionInfo, PhysicalSize, Point, Rect,
    Size, WidgetNode, WidgetNodeRequests, WidgetNodeType, VG,
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

    let mut test_background_node_ref = app_window.add_background_node(
        window_logical_size,
        0,
        Point::new(0.0, 0.0),
        true,
        Box::new(TestBackgroundNode {}),
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

                    window.request_redraw();
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
                }

                window.request_redraw();
            }
            _ => {}
        },
        Event::RedrawRequested(window_id) if window_id == window.id() => {
            app_window.render(window_size, Color::rgb(30, 30, 30));

            gl_surface.swap_buffers(&current_gl_context).unwrap();
            window.request_redraw();
        }
        Event::MainEventsCleared => {}
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
        use firewheel::vg::{Color, Paint, Path};

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
            region.physical_rect.y2() as f32,
            region.physical_rect.pos.x as f32,
            region.physical_rect.y2() as f32 - (19.0 * region.scale_factor.as_f32()),
            Color::rgb(64, 64, 64),
            Color::rgb(50, 50, 50),
        );

        let mut border_paint = Paint::color(Color::rgb(6, 6, 6));
        border_paint.set_line_width((1.0 * region.scale_factor.as_f32()).round());

        vg.fill_path(&mut path, &gradient_paint);
        vg.stroke_path(&mut path, &border_paint);
    }
}

/*
enum ButtonState {
    Idle,
    Hovered,
    Active,
}

struct TestLabelButton {
    font_size_pts: f32,
    label: String,
    padding_lr: f32,
    padding_tb: f32,
    size: Size,

    state: ButtonState,
}

impl TestLabelButton {
    pub fn new(
        font_size_pts: f32,
        label: String,
        padding_lr: f32,
        padding_tb: f32,
        vg: &VG,
    ) -> Self {
        Self {
            font_size_pts,
            label,
            padding_lr,
            padding_tb,
            state: ButtonState::Idle,
        }
    }

    pub fn size(&self) -> Size {
        self.size
    }

    fn compute_size(
        font_size_pts: f32,
        label: &str,
        padding_lr: f32,
        padding_tb: f32,
        vg: &VG,
    ) -> Size {
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
        println!("region changed");
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
    fn paint(&mut self, vg: &mut VG, region: &PaintRegionInfo) {}
}
*/
