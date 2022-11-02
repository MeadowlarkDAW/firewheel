use firewheel::event::{InputEvent, PointerEvent};
use firewheel::vg::{Color, Paint, Path};
use firewheel::{
    Anchor, AppWindow, BackgroundNode, PaintRegionInfo, ParentAnchorType, PhysicalSize, Point,
    RegionInfo, VG,
};
use glutin::config::{ConfigSurfaceTypes, ConfigTemplateBuilder, GlConfig};
use glutin::context::{ContextApi, ContextAttributesBuilder, NotCurrentGlContextSurfaceAccessor};
use glutin::display::{Display, DisplayApiPreference};
use glutin::prelude::{GlDisplay, GlSurface};
use glutin::surface::{SurfaceAttributesBuilder, SwapInterval, WindowSurface};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use std::any::Any;
use std::num::NonZeroU32;
use std::rc::Rc;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

mod label_button;
use label_button::{LabelButton, LabelButtonEvent, LabelButtonStyle};

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
        AppWindow::<MyAction>::new_from_glutin_display(window.scale_factor().into(), &gl_display);

    let mut window_size = PhysicalSize::new(window.inner_size().width, window.inner_size().height);
    let mut scale_factor = window.scale_factor().into();
    let window_logical_size = window_size.to_logical(scale_factor);
    let mut action_queue = Vec::new();

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

    let mut buttom_msg_i: usize = 0;

    let label_button_style = Rc::new(LabelButtonStyle::default());
    let test_button = LabelButton::new(
        BUTTON_MESSAGES[buttom_msg_i].into(),
        main_font_id,
        label_button_style.clone(),
        Some(MyAction::ButtonPressed),
        false,
    );
    //let test_button_size = test_button.compute_size(scale_factor, app_window.vg());
    let test_button_size = label_button_style.compute_size(
        BUTTON_MESSAGES[buttom_msg_i],
        main_font_id,
        scale_factor,
        app_window.vg(),
    );
    let mut test_button_ref = app_window
        .add_widget_node(
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
            &mut action_queue,
        )
        .unwrap();

    // --- Run event loop --------------------------------------------------------------

    let mut pointer_event_state = PointerEvent::default();

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
                    let window_logical_size = window_size.to_logical(scale_factor);

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
                            &mut action_queue,
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
                    app_window.set_scale_factor(scale_factor, &mut action_queue);

                    window_size = PhysicalSize::new(new_inner_size.width, new_inner_size.height);
                    let window_logical_size = window_size.to_logical(scale_factor);

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
                            &mut action_queue,
                        )
                        .unwrap();
                }
            }
            WindowEvent::ModifiersChanged(m) => {
                pointer_event_state.modifiers = firewheel::event::from_winit_modifiers(m);
            }
            WindowEvent::CursorMoved { position, .. } => {
                pointer_event_state.update_from_winit_cursor_moved(*position, scale_factor);
                app_window.handle_input_event(
                    &InputEvent::Pointer(pointer_event_state.clone()),
                    &mut action_queue,
                );
            }
            WindowEvent::MouseInput { state, button, .. } => {
                pointer_event_state.update_from_winit_mouse_input(state, button);
                app_window.handle_input_event(
                    &InputEvent::Pointer(pointer_event_state.clone()),
                    &mut action_queue,
                );
            }
            WindowEvent::MouseWheel { delta, phase, .. } => {
                pointer_event_state.update_from_winit_mouse_wheel(delta, phase, scale_factor);
                app_window.handle_input_event(
                    &InputEvent::Pointer(pointer_event_state.clone()),
                    &mut action_queue,
                );
            }
            _ => {}
        },
        Event::RedrawRequested(window_id) if window_id == window.id() => {
            app_window.render(window_size, Color::rgb(30, 30, 30));

            gl_surface.swap_buffers(&current_gl_context).unwrap();
        }
        Event::MainEventsCleared => {
            let mut temp_action_queue = Vec::new();
            std::mem::swap(&mut temp_action_queue, &mut action_queue);

            for action in temp_action_queue.drain(..) {
                match action {
                    MyAction::ButtonPressed => {
                        println!("button pressed!");

                        buttom_msg_i = if buttom_msg_i + 1 >= BUTTON_MESSAGES.len() {
                            0
                        } else {
                            buttom_msg_i + 1
                        };
                        let button_msg = BUTTON_MESSAGES[buttom_msg_i];

                        let test_button_size = label_button_style.compute_size(
                            button_msg,
                            main_font_id,
                            scale_factor,
                            app_window.vg(),
                        );

                        app_window
                            .send_user_event_to_widget(
                                &mut test_button_ref,
                                Box::new(LabelButtonEvent::<MyAction>::SetLabel(button_msg.into())),
                                &mut action_queue,
                            )
                            .unwrap();
                        app_window
                            .modify_widget_region(
                                &mut test_button_ref,
                                Some(test_button_size),
                                None,
                                None,
                                None,
                                &mut action_queue,
                            )
                            .unwrap();
                    }
                }
            }

            temp_action_queue.append(&mut action_queue);
            std::mem::swap(&mut temp_action_queue, &mut action_queue);

            if app_window.is_dirty() {
                window.request_redraw();
            }
        }
        //Event::DeviceEvent { device_id, event } => {}
        _ => {}
    });
}

#[derive(Debug, Clone)]
enum MyAction {
    ButtonPressed,
}

const BUTTON_MESSAGES: [&str; 5] = [
    "Hello World!",
    "Click Me!",
    "It is me. A button.",
    "LOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOONG BOI",
    "sup",
];

struct TestBackgroundNode {}

impl BackgroundNode for TestBackgroundNode {
    fn on_user_event(&mut self, _event: Box<dyn Any>) -> bool {
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
