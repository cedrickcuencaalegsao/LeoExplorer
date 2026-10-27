use iced::widget::svg::{self, Handle, Svg};
use iced::{Color, Element, Theme};

use crate::config;

#[derive(Clone, Copy)]
pub struct Icon {
    name: &'static str,
    bytes: &'static [u8],
    size: f32,
}

impl Icon {
    pub fn new(name: &'static str, bytes: &'static [u8], size: f32) -> Self {
        Self { name, bytes, size }
    }

    pub fn render<'a>(self, hidden: bool, active: bool) -> Svg<'a> {
        let custom = config::icon_color(self.name);

        Svg::new(Handle::from_memory(self.bytes))
            .width(self.size)
            .height(self.size)
            .style(move |theme: &Theme, _status| {
                let palette = theme.palette();
                let base = if active {
                    palette.primary
                } else {
                    custom.unwrap_or(palette.text)
                };
                let color = if hidden { Color { a: base.a * 0.5, ..base } } else { base };
                svg::Style { color: Some(color) }
            })
    }
}

impl<'a, Message: 'a> From<Icon> for Element<'a, Message> {
    fn from(icon: Icon) -> Self {
        icon.render(false, false).into()
    }
}

macro_rules! icon_fn {
    ($name:ident, $file:literal) => {
        pub fn $name(size: f32) -> Icon {
            Icon::new($file, include_bytes!(concat!("../../assets/icons/", $file)), size)
        }
    };
}

icon_fn!(rocket, "rocket.svg");
icon_fn!(house, "house.svg");
icon_fn!(monitor, "monitor.svg");
icon_fn!(files, "files.svg");
icon_fn!(download, "download.svg");
icon_fn!(headphones, "headphones.svg");
icon_fn!(image, "image.svg");
icon_fn!(square_play, "square-play.svg");
icon_fn!(folder, "folder.svg");
icon_fn!(file_text, "file-text.svg");
icon_fn!(hard_drive, "hard-drive.svg");
icon_fn!(usb, "usb.svg");
icon_fn!(eject, "eject.svg");
icon_fn!(moon, "moon.svg");
icon_fn!(sun, "sun.svg");
icon_fn!(lock, "lock.svg");