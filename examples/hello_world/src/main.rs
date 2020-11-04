use goldenrod::{
    texture, Application, Background, Color, Message, Parent, Point, Root,
    Runner, Size, WindowOpenOptions, WindowScalePolicy,
};

#[derive(Debug, Copy, Clone, Hash)]
enum Textures {
    HappyTree,
}

impl Textures {
    pub const ALL: [Textures; 1] = [Textures::HappyTree];
}

impl texture::IdGroup for Textures {}

impl From<Textures> for texture::Handle {
    fn from(texture: Textures) -> Self {
        match texture {
            Textures::HappyTree => texture::Handle::from_1x_only(
                texture::Source::from_path("./happy-tree.png", Point::ORIGIN),
            ),
        }
    }
}

struct HelloWorldExample {}

impl HelloWorldExample {
    fn new(root: &mut Root<Textures>) -> Self {
        root.replace_texture_atlas(&Textures::ALL).unwrap();
        //root.set_background(Background::SolidColor(Color::new(0.02, 0.02, 0.025, 1.0)));
        root.set_background(Background::Texture(Textures::HappyTree));

        Self {}
    }
}

impl Application for HelloWorldExample {
    type CustomMessage = ();
    type TextureIDs = Textures;

    fn on_message(&mut self, message: Message<()>, root: &mut Root<Textures>) {}
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
