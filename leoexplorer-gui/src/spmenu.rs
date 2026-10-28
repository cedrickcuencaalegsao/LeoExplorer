use iced::widget::text::Wrapping;
use iced::widget::{button, column, container, row, text, tooltip, Column};
use iced::{Alignment, Color, Element, Length, Theme};

use crate::icons::{self, Icon};
use crate::spitem::fit_label;

const FONT_SIZE: f32 = 18.0;
const BUTTON_PADDING: f32 = 20.0;
const SPACING: f32 = 12.0;
const LOCK_SIZE: f32 = 16.0;
const TOGGLE_SLOT: f32 = 12.0;
const SAFETY: f32 = 8.0;
const DEFAULT_WIDTH: f32 = 248.0;

pub struct SpMenu<'a, Message> {
    icon: Icon,
    label: String,
    hidden: bool,
    permission_required: bool,
    active: bool,
    expanded: bool,
    expandable: bool,
    width: f32,
    children: Vec<Element<'a, Message>>,
    on_click: Option<Box<dyn Fn() -> Message + 'a>>,
}

impl<'a, Message: 'a> SpMenu<'a, Message> {
    pub fn new(icon: Icon, label: impl Into<String>) -> Self {
        Self {
            icon,
            label: label.into(),
            hidden: false,
            permission_required: false,
            active: false,
            expanded: false,
            expandable: false,
            width: DEFAULT_WIDTH,
            children: Vec::new(),
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

    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }

    pub fn expandable(mut self, expandable: bool) -> Self {
        self.expandable = expandable;
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub fn child(mut self, child: impl Into<Element<'a, Message>>) -> Self {
        self.children.push(child.into());
        self
    }

    pub fn children(
        mut self,
        children: impl IntoIterator<Item = impl Into<Element<'a, Message>>>,
    ) -> Self {
        self.children.extend(children.into_iter().map(Into::into));
        self
    }

    pub fn on_click(mut self, callback: impl Fn() -> Message + 'a) -> Self {
        self.on_click = Some(Box::new(callback));
        self
    }
}

impl<'a, Message: Clone + 'a> From<SpMenu<'a, Message>> for Element<'a, Message> {
    fn from(menu: SpMenu<'a, Message>) -> Self {
        let SpMenu {
            icon,
            label,
            hidden,
            permission_required,
            active,
            expanded,
            expandable,
            width,
            children,
            on_click,
        } = menu;

        let toggle = if !(expandable || !children.is_empty()) {
            ""
        } else if expanded {
            "−"
        } else {
            "+"
        };

        let lock_space = if permission_required { SPACING + LOCK_SIZE } else { 0.0 };
        let budget = width
            - BUTTON_PADDING
            - icon.size()
            - SPACING
            - SPACING
            - TOGGLE_SLOT
            - lock_space
            - SAFETY;
        let (shown, truncated) = fit_label(&label, FONT_SIZE, budget);

        let mut header = row![
            icon.render(hidden, active),
            container(text(shown).size(FONT_SIZE).wrapping(Wrapping::None))
                .width(Length::Fill)
                .clip(true),
        ]
        .spacing(SPACING)
        .align_y(Alignment::Center);

        if permission_required {
            header = header.push(icons::lock(LOCK_SIZE).render(hidden, active));
        }

        header = header.push(text(toggle).size(FONT_SIZE));

        let mut head = button(header)
            .width(Length::Fill)
            .style(move |theme: &Theme, status| {
                let mut style = button::text(theme, status);
                if active {
                    style.text_color = theme.palette().primary;
                }
                if hidden {
                    style.text_color = Color { a: 0.5, ..style.text_color };
                }
                style
            });

        if let Some(callback) = on_click {
            head = head.on_press_with(callback);
        }

        let head: Element<'a, Message> = if truncated {
            tooltip(
                head,
                container(text(label).size(13))
                    .padding(6)
                    .style(container::rounded_box),
                tooltip::Position::FollowCursor,
            )
            .into()
        } else {
            head.into()
        };

        let mut menu = Column::new().spacing(2).push(head);

        if expanded {
            for child in children {
                menu = menu.push(child);
            }
        }

        column![menu].into()
    }
}