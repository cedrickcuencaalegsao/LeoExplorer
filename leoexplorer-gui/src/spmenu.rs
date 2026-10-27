use iced::widget::{button, column, horizontal_space, row, text, Column};
use iced::{Alignment, Color, Element, Length, Theme};

use crate::icons::{self, Icon};

pub struct SpMenu<'a, Message> {
    icon: Icon,
    label: String,
    hidden: bool,
    permission_required: bool,
    active: bool,
    expanded: bool,
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
            children,
            on_click,
        } = menu;

        let toggle = match (children.is_empty(), expanded) {
            (true, _) => "",
            (false, true) => "−",
            (false, false) => "+",
        };

        let mut header = row![icon.render(hidden, active), text(label).size(18)]
            .spacing(12)
            .align_y(Alignment::Center)
            .push(horizontal_space());

        if permission_required {
            header = header.push(icons::lock(16.0).render(hidden, active));
        }

        header = header.push(text(toggle).size(18));

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

        let mut menu = Column::new().spacing(2).push(head);

        if expanded {
            for child in children {
                menu = menu.push(child);
            }
        }

        column![menu].into()
    }
}