use crate::{
    root::RootState, settings, Application, Message, Root, Settings, Size,
};
use baseview::{
    Event, Parent, Window, WindowHandle, WindowHandler, WindowOpenOptions,
    WindowScalePolicy,
};

pub struct Runner<A: Application + 'static + Send> {
    user_app: A,
    root_state: RootState,
}

impl<A: Application + 'static + Send> Runner<A> {
    /// Open a new window
    pub fn open<B>(settings: Settings, build: B) -> WindowHandle
    where
        B: FnOnce(&mut Root) -> A,
        B: Send + 'static,
    {
        let scale_policy = match settings.window.scale {
            settings::ScalePolicy::SystemScaleFactor => {
                WindowScalePolicy::SystemScaleFactor
            }
            settings::ScalePolicy::ScaleFactor(scale) => {
                WindowScalePolicy::ScaleFactor(scale)
            }
        };

        let window_options = WindowOpenOptions {
            title: settings.window.title,
            size: settings.window.size.into(),
            scale: scale_policy,
            parent: Parent::None,
        };

        Window::open(window_options, move |window: &mut Window| -> Runner<A> {
            let mut root_state = RootState::new(window);
            let mut root = Root::new(&mut root_state, window);

            let user_app = build(&mut root);

            Runner {
                user_app,
                root_state,
            }
        })
    }
}

impl<A: Application + 'static + Send> WindowHandler for Runner<A> {
    type Message = Message<A::CustomMessage>;

    fn on_frame(&mut self) {
        self.root_state.render();
    }

    fn on_event(&mut self, window: &mut Window, event: Event) {
        match &event {
            Event::Window(e) => match e {
                baseview::WindowEvent::Resized(window_info) => {
                    let physical_size = Size::<u16>::new(
                        window_info.physical_size().width as u16,
                        window_info.physical_size().height as u16,
                    );

                    self.root_state
                        .window_resized(physical_size, window_info.scale());
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn on_message(&mut self, window: &mut Window, message: Self::Message) {
        let mut root = Root::new(&mut self.root_state, window);

        self.user_app.on_message(message, &mut root);
    }
}
