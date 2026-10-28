use iced::widget::text::Wrapping;
use iced::widget::{button, container, row, text, tooltip};
use iced::{Alignment, Color, Element, Length, Padding, Theme};

use crate::icons::{self, Icon};

const FONT_SIZE: f32 = 15.0;
const BUTTON_PADDING: f32 = 20.0;
const SPACING: f32 = 10.0;
const LOCK_SIZE: f32 = 14.0;
const SAFETY: f32 = 8.0;
const DEFAULT_WIDTH: f32 = 248.0;

fn glyph_width(c: char, size: f32) -> f32 {
    let em = match c {
        ' ' => 0.28,
        'i' | 'j' | 'l' | 't' | 'f' | 'I' | '.' | ',' | ':' | ';' | '\'' | '!' | '|' => 0.32,
        'm' | 'w' | 'M' | 'W' => 0.86,
        c if c.is_ascii_uppercase() => 0.68,
        c if c.is_ascii_digit() => 0.56,
        c if c.is_ascii() => 0.55,
        _ => 1.0,
    };
    em * size
}

pub(crate) fn fit_label(label: &str, size: f32, max_width: f32) -> (String, bool) {
    let total: f32 = label.chars().map(|c| glyph_width(c, size)).sum();

    if total <= max_width {
        return (label.to_string(), false);
    }

    let ellipsis = glyph_width('.', size) * 3.0;
    let mut width = 0.0;
    let mut fitted = String::new();

    for c in label.chars() {
        let w = glyph_width(c, size);
        if width + w + ellipsis > max_width {
            break;
        }
        width += w;
        fitted.push(c);
    }

    let mut fitted = fitted.trim_end().to_string();
    fitted.push_str("...");
    (fitted, true)
}

pub struct SpItem<'a, Message> {
    icon: Icon,
    label: String,
    hidden: bool,
    permission_required: bool,
    indent: f32,
    width: f32,
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
            width: DEFAULT_WIDTH,
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

    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
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
            width,
            on_click,
        } = item;

        let lock_space = if permission_required { SPACING + LOCK_SIZE } else { 0.0 };
        let budget = width - indent - BUTTON_PADDING - icon.size() - SPACING - lock_space - SAFETY;
        let (shown, truncated) = fit_label(&label, FONT_SIZE, budget);

        let mut content = row![
            icon.render(hidden, false),
            container(text(shown).size(FONT_SIZE).wrapping(Wrapping::None))
                .width(Length::Fill)
                .clip(true),
        ]
        .spacing(SPACING)
        .align_y(Alignment::Center);

        if permission_required {
            content = content.push(icons::lock(LOCK_SIZE).render(hidden, false));
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

        let entry: Element<'a, Message> = if truncated {
            tooltip(
                entry,
                container(text(label).size(13))
                    .padding(6)
                    .style(container::rounded_box),
                tooltip::Position::FollowCursor,
            )
            .into()
        } else {
            entry.into()
        };

        container(entry)
            .padding(Padding { left: indent, ..Padding::ZERO })
            .into()
    }
}