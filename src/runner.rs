use crate::{
    renderer::Renderer, settings, settings::ScalePolicy, Application, Message,
    PhySize, Root, Settings, Size, Tree,
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
    scale_policy: ScalePolicy,
}

impl<A: Application + 'static + Send> Runner<A> {
    /// Open a new window
    pub fn open<B>(settings: Settings, build: B) -> WindowHandle
    where
        B: FnOnce(&mut Root<A::TextureIDs>) -> A,
        B: Send + 'static,
    {
        let scale_policy = settings.window.scale;

        let logical_width = settings.window.logical_size.width() as f64;
        let logical_height = settings.window.logical_size.height() as f64;

        let window_options = WindowOpenOptions {
            title: settings.window.title,
            size: baseview::Size::new(logical_width, logical_height),
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

                let physical_size = PhySize::new(
                    (logical_width * scale) as i32,
                    (logical_height * scale) as i32,
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
                    scale_policy,
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
        let render_info = self.widget_tree.get_render_info();

        self.renderer.render();
    }

    fn on_event(&mut self, _window: &mut Window, event: Event) {
        match &event {
            Event::Window(e) => match e {
                baseview::WindowEvent::Resized(window_info) => {
                    let physical_size = PhySize::new(
                        window_info.physical_size().width as i32,
                        window_info.physical_size().height as i32,
                    );

                    let scale = match self.scale_policy {
                        ScalePolicy::ScaleFactor(scale) => scale,
                        ScalePolicy::SystemScaleFactor => window_info.scale(),
                    };

                    self.renderer.resize(physical_size, scale);
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
