use goldenrod::{
    hash_id, settings, Application, Background, Color, Message, Parent, Point,
    Root, Runner, Settings, Size, Texture, TextureSource,
};

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
enum TextureID {
    Back,
    Knob,
    TestVSlider,
    TestHSlider,
}

impl TextureID {
    fn all() -> [(TextureID, Texture); 4] {
        [
            (
                TextureID::Back,
                Texture::res_1x(TextureSource::path(
                    "./examples/petal_drive/images/1x/back.png",
                    Point::ORIGIN,
                )),
            ),
            (
                TextureID::Knob,
                Texture::res_1x(TextureSource::path(
                    "./examples/petal_drive/images/1x/knob.png",
                    Point::new(40.0, 40.0),
                )),
            ),
            (
                TextureID::TestVSlider,
                Texture::res_1x(TextureSource::path(
                    "./examples/petal_drive/images/1x/test_v_slider.png",
                    Point::new(10.0, 22.0),
                )),
            ),
            (
                TextureID::TestHSlider,
                Texture::res_1x(TextureSource::path(
                    "./examples/petal_drive/images/1x/test_h_slider.png",
                    Point::new(22.0, 10.0),
                )),
            ),
        ]
    }
}

struct HelloWorldExample {}

impl HelloWorldExample {
    fn new(root: &mut Root) -> Self {
        root.replace_texture_atlas(&TextureID::all()).unwrap();
        root.set_background(Background::Texture(hash_id(TextureID::Back)));

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
            title: "goldenrod: petal_drive".into(),
            size: Size::new(485, 285),
            scale: settings::ScalePolicy::SystemScaleFactor,
        },
    };

    let handle = Runner::open(settings, |root| -> HelloWorldExample {
        HelloWorldExample::new(root)
    });

    handle.app_run_blocking();
}
