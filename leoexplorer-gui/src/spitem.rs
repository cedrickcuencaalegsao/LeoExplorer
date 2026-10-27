use iced::widget::{button, container, horizontal_space, row, text};
use iced::{Alignment, Color, Element, Length, Padding, Theme};

use crate::icons::{self, Icon};

pub struct SpItem<'a, Message> {
    icon: Icon,
    label: String,
    hidden: bool,
    permission_required: bool,
    indent: f32,
    on_click: Option<Box<dyn Fn() -> Message + 'a>>,
}

impl<'a, Message: 'a> SpItem<'a, Message> {
    pub fn new(icon: Icon, label: impl Into<String>) -> Self {
        Self {
            icon,
            label: label.into(),
            hidden: false,
            permission_required: false,
            indent: 24.0,
            on_click: None,
        }
    }

    pub fn hidden(mut self, hidden: bool) -> Self {
        self.hidden = hidden;
        self
    }

    pub fn permission_required(mut self, required: bool) -> Self {
        self.permission_required = required;
        self
    }

    pub fn indent(mut self, indent: f32) -> Self {
        self.indent = indent;
        self
    }

    pub fn on_click(mut self, callback: impl Fn() -> Message + 'a) -> Self {
        self.on_click = Some(Box::new(callback));
        self
    }
}

impl<'a, Message: Clone + 'a> From<SpItem<'a, Message>> for Element<'a, Message> {
    fn from(item: SpItem<'a, Message>) -> Self {
        let SpItem {
            icon,
            label,
            hidden,
            permission_required,
            indent,
            on_click,
        } = item;

        let mut content = row![icon.render(hidden, false), text(label).size(15)]
            .spacing(10)
            .align_y(Alignment::Center);

        if permission_required {
            content = content
                .push(horizontal_space())
                .push(icons::lock(14.0).render(hidden, false));
        }

        let mut entry = button(content)
            .width(Length::Fill)
            .style(move |theme: &Theme, status| {
                let mut style = button::text(theme, status);
                if hidden {
                    style.text_color = Color { a: 0.5, ..style.text_color };
                }
                style
            });

        if let Some(callback) = on_click {
            entry = entry.on_press_with(callback);
        }

        container(entry)
            .padding(Padding { left: indent, ..Padding::ZERO })
            .into()
    }
}