use goldenrod::{
    hash_id, settings, Application, Background, Color, Message, Parent, Root,
    Runner, Settings, Size, Texture, TextureSource,
};

#[derive(Debug, Copy, Clone, Hash)]
enum TextureIDs {
    HappyTree,
}

impl TextureIDs {
    fn all() -> [(TextureIDs, Texture); 1] {
        [(
            TextureIDs::HappyTree,
            Texture::res_1x(TextureSource::path("./happy-tree.png", None)),
        )]
    }
}

struct HelloWorldExample {}

impl HelloWorldExample {
    fn new(root: &mut Root) -> Self {
        root.replace_texture_atlas(&TextureIDs::all()).unwrap();
        //root.set_background(Background::SolidColor(Color::new(0.02, 0.02, 0.025, 1.0)));
        root.set_background(Background::Texture(hash_id(
            TextureIDs::HappyTree,
        )));

        Self {}
    }
}

impl Application for HelloWorldExample {
    type CustomMessage = ();

    fn on_message(&mut self, message: Message<()>, root: &mut Root) {}
}

fn main() {
    let settings = Settings {
        window: settings::Window {
            title: "goldenrod: hello world".into(),
            size: Size::new(256, 256),
            scale: settings::ScalePolicy::SystemScaleFactor,
        },
    };

    let handle = Runner::open(settings, |root| -> HelloWorldExample {
        HelloWorldExample::new(root)
    });

    handle.app_run_blocking();
}
