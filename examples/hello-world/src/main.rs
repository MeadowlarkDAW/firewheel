use raw_gl_context::{GlConfig, GlContext};
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
    gl::load_with(|s| context.get_proc_address(s) as _);
    context.make_not_current();

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() => match event {
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            WindowEvent::Resized(physical_size) => {}
            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {}
            _ => {}
        },
        Event::RedrawRequested(window_id) if window_id == window.id() => {
            context.make_current();

            unsafe {
                gl::ClearColor(0.175, 0.175, 0.175, 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT);
            }

            context.swap_buffers();
            context.make_not_current();
        }
        Event::MainEventsCleared => {
            window.request_redraw();
        }
        _ => {}
    });
}
