use crate::{
    settings, settings::ScalePolicy, wgpu_renderer::Renderer, Application,
    Message, Root, Settings, Size, Tree,
};
use baseview::{
    Event, Parent, Window, WindowHandle, WindowHandler, WindowOpenOptions,
    WindowScalePolicy,
};
use futures::executor::block_on;

pub struct Runner<A: Application + 'static + Send> {
    user_app: A,
    widget_tree: Tree<A::TextureIDs, A::WidgetIDs>,
    renderer: Renderer<A::TextureIDs>,
}

impl<A: Application + 'static + Send> Runner<A> {
    /// Open a new window
    pub fn open<B>(settings: Settings, build: B) -> WindowHandle
    where
        B: FnOnce(&mut Root<A::TextureIDs>) -> A,
        B: Send + 'static,
    {
        let scale_policy = settings.window.scale;

        let logical_width = settings.window.logical_size.width as f64;
        let logical_height = settings.window.logical_size.height as f64;

        let window_options = WindowOpenOptions {
            title: settings.window.title,
            size: settings.window.logical_size.into(),
            scale: match settings.window.scale {
                settings::ScalePolicy::SystemScaleFactor => {
                    WindowScalePolicy::SystemScaleFactor
                }
                settings::ScalePolicy::ScaleFactor(scale) => {
                    WindowScalePolicy::ScaleFactor(scale)
                }
            },
            parent: Parent::None,
        };

        Window::open(
            window_options,
            move |window: &mut Window<'_>| -> Runner<A> {
                // Assume scale for now until there is an event with a new one.
                let scale = match scale_policy {
                    ScalePolicy::ScaleFactor(scale) => scale,
                    ScalePolicy::SystemScaleFactor => 1.0,
                };

                let physical_size = Size::<u16>::new(
                    (logical_width * scale) as u16,
                    (logical_height * scale) as u16,
                );

                let mut renderer =
                    block_on(Renderer::new(window, physical_size, scale))
                        .unwrap();

                let mut root = Root::new(window, &mut renderer);

                let user_app = build(&mut root);

                Runner {
                    user_app,
                    widget_tree: Tree::new(),
                    renderer,
                }
            },
        )
    }
}

impl<A: Application + 'static + Send> WindowHandler for Runner<A> {
    type Message = Message<A::CustomMessage>;

    fn on_frame(&mut self) {
        // Construct the current widget tree.
        self.widget_tree.start_tree_construction();
        self.user_app.view(&mut self.widget_tree);

        // TODO: Check for duplicate widgets with the same id.

        // Retrieve any rendering changes.
        let render_info = self
            .widget_tree
            .get_render_info(self.renderer.needs_full_redraw());

        self.renderer.render();
    }

    fn on_event(&mut self, _window: &mut Window, event: Event) {
        match &event {
            Event::Window(e) => match e {
                baseview::WindowEvent::Resized(window_info) => {
                    let physical_size = Size::<u16>::new(
                        window_info.physical_size().width as u16,
                        window_info.physical_size().height as u16,
                    );

                    self.renderer.resize(physical_size, window_info.scale());
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn on_message(&mut self, window: &mut Window, message: Self::Message) {
        let mut root = Root::new(window, &mut self.renderer);

        self.user_app.on_message(message, &mut root);
    }
}
