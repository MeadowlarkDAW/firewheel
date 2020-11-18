use goldenrod::{
    settings, Application, Background, Color, IdGroup, Message, Parent, Point,
    Root, Runner, Settings, Size, Texture, TextureSource, Tree,
};

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
enum TextureID {
    Back,
    Knob,
    TestVSlider,
    TestHSlider,
}

impl IdGroup for TextureID {}

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

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
enum WidgetID {
    Test,
}

impl IdGroup for WidgetID {}

struct PetalDriveGUI {}

impl PetalDriveGUI {
    fn new(root: &mut Root<TextureID>) -> Self {
        root.replace_texture_atlas(&TextureID::all()).unwrap();
        root.set_background(Background::Texture(TextureID::Back));

        Self {}
    }
}

impl Application for PetalDriveGUI {
    type TextureIDs = TextureID;
    type WidgetIDs = WidgetID;

    type CustomMessage = ();

    fn on_message(&mut self, message: Message<()>, root: &mut Root<TextureID>) {
    }

    fn view(&self, tree: &mut Tree<TextureID, WidgetID>) {}
}

fn main() {
    let settings = Settings {
        window: settings::Window {
            title: "goldenrod: petal_drive".into(),
            size: Size::new(485, 285),
            scale: settings::ScalePolicy::SystemScaleFactor,
        },
    };

    let handle = Runner::open(settings, |root| -> PetalDriveGUI {
        PetalDriveGUI::new(root)
    });

    handle.app_run_blocking();
}
