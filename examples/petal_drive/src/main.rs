use goldenrod::{
    settings, texture, Application, Background, Message, Point, Root, Runner,
    Settings, Size,
};

#[derive(Default)]
struct PetalDriveGUI {
    tex_back: texture::Handle,
    tex_knob: texture::Handle,
    tex_vslider: texture::Handle,
    tex_hslider: texture::Handle,
}

impl PetalDriveGUI {
    fn new(root: &mut Root) -> Self {
        let mut gui = Self::default();

        root.new_texture_atlas(&mut gui.load_textures()).unwrap();
        root.set_background(Background::Texture(gui.tex_back.clone()));

        gui
    }

    fn load_textures(&mut self) -> Vec<texture::Loader> {
        vec![
            texture::Loader::res_1x(
                &mut self.tex_back,
                texture::Source::path(
                    "./examples/petal_drive/images/1x/back.png",
                    Point::ORIGIN,
                ),
            ),
            texture::Loader::res_1x(
                &mut self.tex_knob,
                texture::Source::path(
                    "./examples/petal_drive/images/1x/knob.png",
                    Point::new(40.0, 40.0),
                ),
            ),
            texture::Loader::res_1x(
                &mut self.tex_vslider,
                texture::Source::path(
                    "./examples/petal_drive/images/1x/test_v_slider.png",
                    Point::new(10.0, 22.0),
                ),
            ),
            texture::Loader::res_1x(
                &mut self.tex_hslider,
                texture::Source::path(
                    "./examples/petal_drive/images/1x/test_h_slider.png",
                    Point::new(22.0, 10.0),
                ),
            ),
        ]
    }
}

impl Application for PetalDriveGUI {
    type CustomMessage = ();

    fn on_message(&mut self, _message: Message<()>, _root: &mut Root) {}
}

fn main() {
    let settings = Settings {
        window: settings::Window {
            title: "goldenrod: petal_drive".into(),
            logical_size: Size::new(485.0, 285.0),
            scale: settings::ScalePolicy::SystemScaleFactor,
        },
    };

    let handle = Runner::open(settings, |root| -> PetalDriveGUI {
        PetalDriveGUI::new(root)
    });

    handle.app_run_blocking();
}
