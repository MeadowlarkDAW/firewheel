use crate::{Application, Canvas, Message, Size};
use baseview::{Event, Window, WindowHandle, WindowHandler, WindowOpenOptions};

pub struct Runner<A: Application + 'static + Send> {
    user_app: A,
    canvas: Canvas,
}

impl<A: Application + 'static + Send> Runner<A> {
    /// Open a new window
    pub fn open<B>(settings: WindowOpenOptions, build: B) -> WindowHandle
    where
        B: FnOnce(&mut Canvas) -> A,
        B: Send + 'static,
    {
        Window::open(settings, move |window: &mut Window| -> Runner<A> {
            let mut canvas = Canvas::new(window);

            let user_app = build(&mut canvas);

            Runner { user_app, canvas }
        })
    }
}

impl<A: Application + 'static + Send> WindowHandler for Runner<A> {
    type Message = Message<A::CustomMessage>;

    fn on_frame(&mut self) {
        self.user_app.on_frame(&mut self.canvas);

        self.canvas.renderer.render();
    }

    fn on_event(&mut self, _window: &mut Window, event: Event) {
        match &event {
            Event::Window(e) => match e {
                baseview::WindowEvent::Resized(window_info) => {
                    let physical_size = Size::new(
                        window_info.physical_size().width as f32,
                        window_info.physical_size().height as f32,
                    );

                    self.canvas
                        .renderer
                        .resize(physical_size, window_info.scale());
                }
                _ => {}
            },
            _ => {}
        }

        self.user_app.on_raw_event(event, &mut self.canvas);
    }

    fn on_message(&mut self, _window: &mut Window, message: Self::Message) {
        self.user_app.on_message(message, &mut self.canvas);
    }
}
