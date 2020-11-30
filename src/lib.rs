use baseview::{Event, Window, WindowHandler, WindowScalePolicy};

surfman::declare_surfman!();

mod renderer;
pub(crate) use renderer::Renderer;

pub struct Runner {
    renderer: Renderer,
}

impl Runner {
    pub fn run() {
        use raw_window_handle::HasRawWindowHandle;

        let window_open_options = baseview::WindowOpenOptions {
            title: "baseview".into(),
            size: baseview::Size::new(512.0, 512.0),
            scale: WindowScalePolicy::SystemScaleFactor,
            parent: baseview::Parent::None,
        };

        let handle =
            Window::open(window_open_options, |window: &mut Window<'_>| {
                let renderer = Renderer::new(window.raw_window_handle());

                Runner { renderer }
            });
        handle.app_run_blocking();
    }
}

impl WindowHandler for Runner {
    type Message = ();

    fn on_frame(&mut self) {
        self.renderer.render(true);
    }

    fn on_event(&mut self, _window: &mut Window, _event: Event) {}

    fn on_message(&mut self, _window: &mut Window, _message: Self::Message) {}
}
