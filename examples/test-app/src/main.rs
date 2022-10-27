use firewheel::{AppWindow, BackgroundNode, PaintRegionInfo, PhysicalSize, Point, VG};
use raw_gl_context::{GlConfig, GlContext};
use std::any::Any;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let gl_config = GlConfig {
        vsync: true,
        ..Default::default()
    };

    let context = GlContext::create(&window, gl_config).unwrap();
    context.make_current();
    let mut app_window = unsafe {
        AppWindow::<()>::new_from_function(window.scale_factor().into(), |s| {
            context.get_proc_address(s) as _
        })
    };
    context.make_not_current();

    let mut window_size = PhysicalSize::new(window.inner_size().width, window.inner_size().height);
    let mut scale_factor = window.scale_factor().into();
    let mut window_logical_size = window_size.to_logical(scale_factor);
    let mut msg_out_queue = Vec::new();

    let mut test_background_node = Some(app_window.add_background_node(
        window_logical_size,
        0,
        Point::new(0.0, 0.0),
        true,
        Box::new(TestBackgroundNode {}),
    ));

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() => match event {
            WindowEvent::CloseRequested => {
                *control_flow = ControlFlow::Exit;

                app_window.remove_background_node(test_background_node.take().unwrap());
            }
            WindowEvent::Resized(physical_size) => {
                window_size = PhysicalSize::new(physical_size.width, physical_size.height);
                window_logical_size = window_size.to_logical(scale_factor);

                app_window.set_background_node_size(
                    test_background_node.as_mut().unwrap(),
                    window_logical_size,
                );
            }
            WindowEvent::ScaleFactorChanged {
                scale_factor: window_scale_factor,
                new_inner_size,
            } => {
                scale_factor = (*window_scale_factor).into();
                app_window.set_scale_factor(scale_factor, &mut msg_out_queue);

                window_size = PhysicalSize::new(new_inner_size.width, new_inner_size.height);
                window_logical_size = window_size.to_logical(scale_factor);

                app_window.set_background_node_size(
                    test_background_node.as_mut().unwrap(),
                    window_logical_size,
                );
            }
            _ => {}
        },
        Event::RedrawRequested(window_id) if window_id == window.id() => {
            context.make_current();

            app_window.render([0.06, 0.06, 0.06, 1.0]);

            context.swap_buffers();
            context.make_not_current();
        }
        Event::MainEventsCleared => {
            window.request_redraw();
        }
        _ => {}
    });
}

struct TestBackgroundNode {}

impl BackgroundNode for TestBackgroundNode {
    fn on_user_event(&mut self, event: Box<dyn Any>) -> bool {
        false
    }

    fn paint(&mut self, vg: &mut VG, region: &PaintRegionInfo) {
        use firewheel::vg::{Color, Paint, Path};

        let mut path = Path::new();
        path.rounded_rect(
            region.physical_rect.pos.x as f32,
            region.physical_rect.pos.y as f32,
            region.physical_rect.size.width as f32,
            region.physical_rect.size.height as f32,
            10.0 * region.scale_factor.as_f32(),
        );

        let paint = Paint::linear_gradient(
            region.physical_rect.pos.x as f32,
            region.physical_rect.pos.y as f32,
            region.physical_rect.x2() as f32,
            region.physical_rect.y2() as f32,
            Color::rgb(0, 255, 0),
            Color::rgb(0, 0, 255),
        );

        vg.fill_path(&mut path, &paint);
    }
}
