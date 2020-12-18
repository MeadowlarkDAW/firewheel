use crate::{
    node, renderer::Renderer, settings, settings::ScalePolicy, Application,
    PhySize, Root, Settings,
};
use baseview::{
    AppRunner, Event, Parent, Window, WindowHandler, WindowOpenOptions,
    WindowScalePolicy,
};
use futures::executor::block_on;

pub struct Runner<A: Application + 'static> {
    _user_app: A,
    renderer: Renderer,
    scale_policy: ScalePolicy,
    tree: node::Tree,
}

impl<A: Application + 'static> Runner<A> {
    /// Open a new window
    pub fn open<B>(settings: Settings, build: B) -> Option<AppRunner>
    where
        B: FnOnce(&mut Root) -> A,
        B: Send + 'static,
    {
        let scale_policy = settings.window.scale;

        let logical_width = settings.window.logical_size.width() as f64;
        let logical_height = settings.window.logical_size.height() as f64;

        let antialiasing = settings.antialiasing;

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

                let mut renderer = block_on(Renderer::new(
                    window,
                    physical_size,
                    scale,
                    antialiasing,
                ))
                .unwrap();

                let mut root = Root::new(window, &mut renderer);

                let mut user_app = build(&mut root);

                let tree = node::Tree::new(user_app.load_nodes());

                // TODO: alert renderer of texture handles

                Runner {
                    _user_app: user_app,
                    renderer,
                    scale_policy,
                    tree,
                }
            },
        )
    }
}

// Safe because `WindowHandler` is effectively treated like a single-threaded system.
unsafe impl<A: Application + 'static> Send for Runner<A> {}

impl<A: Application + 'static> WindowHandler for Runner<A> {
    fn on_frame(&mut self) {
        /*
        // Construct the current widget tree.
        self.widget_tree.start_tree_construction();
        self._user_app.view(&mut self.widget_tree);

        // TODO: Check for duplicate widgets with the same id.

        // Retrieve any rendering changes.
        let render_info = self.widget_tree.get_render_info();
        */

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
}
