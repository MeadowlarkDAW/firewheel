use goldenrod::{
    Application, Canvas, Message, Parent, Point, Runner, Size, TextureHandle,
    TextureSource, Window, WindowOpenOptions, WindowScalePolicy,
};

#[derive(Debug, Copy, Clone)]
enum Textures {
    HappyTree,
}

impl Textures {
    pub const ALL: [Textures; 1] = [Textures::HappyTree];
}

impl From<Textures> for TextureHandle {
    fn from(texture: Textures) -> Self {
        match texture {
            Textures::HappyTree => TextureHandle::from_1x(
                TextureSource::from_path("./happy-tree.png", Point::ORIGIN),
            ),
        }
    }
}

struct HelloWorldExample {}

impl HelloWorldExample {
    fn new(canvas: &mut Canvas) -> Self {
        canvas.replace_texture_atlas(&Textures::ALL).unwrap();

        Self {}
    }
}

impl Application for HelloWorldExample {
    type CustomMessage = ();

    fn on_message(
        &mut self,
        message: Message<Self::CustomMessage>,
        canvas: &mut Canvas,
    ) {
    }
}

fn main() {
    let options = WindowOpenOptions {
        title: "goldenrod: hello world".into(),
        size: Size::new(512.0, 512.0).into(),
        scale: WindowScalePolicy::SystemScaleFactor,
        parent: Parent::None,
    };

    let handle = Runner::open(options, |canvas| -> HelloWorldExample {
        HelloWorldExample::new(canvas)
    });

    handle.app_run_blocking();
}
