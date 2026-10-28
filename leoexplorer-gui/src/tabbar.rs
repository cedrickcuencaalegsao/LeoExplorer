use std::rc::Rc;

use iced::border::Radius;
use iced::widget::text::Wrapping;
use iced::widget::{button, container, responsive, row, text, vertical_rule, Row};
use iced::{Alignment, Background, Border, Color, Element, Length, Padding, Size, Theme};

use crate::icons;

const DEFAULT_MAX_TAB_WIDTH: f32 = 300.0;
const MIN_TAB_WIDTH: f32 = 60.0;
const BAR_HEIGHT: f32 = 40.0;
const PLUS_SLOT: f32 = 36.0;

type Callback<'a, Message> = Rc<dyn Fn() -> Message + 'a>;

struct TabItem<'a, Message> {
    title: String,
    on_select: Callback<'a, Message>,
    on_close: Callback<'a, Message>,
}

pub struct TabBar<'a, Message> {
    tabs: Vec<TabItem<'a, Message>>,
    active: usize,
    max_tab_width: f32,
    on_new: Option<Callback<'a, Message>>,
}

impl<'a, Message: 'a> TabBar<'a, Message> {
    pub fn new(active: usize) -> Self {
        Self {
            tabs: Vec::new(),
            active,
            max_tab_width: DEFAULT_MAX_TAB_WIDTH,
            on_new: None,
        }
    }

    pub fn max_tab_width(mut self, width: f32) -> Self {
        self.max_tab_width = width;
        self
    }

    pub fn tab(
        mut self,
        title: impl Into<String>,
        on_select: impl Fn() -> Message + 'a,
        on_close: impl Fn() -> Message + 'a,
    ) -> Self {
        self.tabs.push(TabItem {
            title: title.into(),
            on_select: Rc::new(on_select),
            on_close: Rc::new(on_close),
        });
        self
    }

    pub fn on_new(mut self, callback: impl Fn() -> Message + 'a) -> Self {
        self.on_new = Some(Rc::new(callback));
        self
    }
}

fn bar_style(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();
    container::Style {
        background: Some(Background::Color(palette.background.weak.color)),
        ..container::Style::default()
    }
}

fn tab_style(active: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |theme, status| {
        let palette = theme.extended_palette();

        let background = if active {
            Some(Background::Color(palette.background.base.color))
        } else if matches!(status, button::Status::Hovered | button::Status::Pressed) {
            Some(Background::Color(Color {
                a: 0.5,
                ..palette.background.strong.color
            }))
        } else {
            None
        };

        button::Style {
            background,
            text_color: palette.background.base.text,
            border: Border {
                radius: Radius {
                    top_left: 10.0,
                    top_right: 10.0,
                    bottom_right: 0.0,
                    bottom_left: 0.0,
                },
                ..Border::default()
            },
            ..button::Style::default()
        }
    }
}

fn round_style(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.extended_palette();

    let background = match status {
        button::Status::Hovered | button::Status::Pressed => Some(Background::Color(Color {
            a: 0.6,
            ..palette.background.strong.color
        })),
        _ => None,
    };

    button::Style {
        background,
        text_color: palette.background.base.text,
        border: Border {
            radius: 12.0.into(),
            ..Border::default()
        },
        ..button::Style::default()
    }
}

fn build_strip<'a, Message: Clone + 'a>(
    tabs: &[TabItem<'a, Message>],
    active: usize,
    max_tab_width: f32,
    on_new: Option<&Callback<'a, Message>>,
    available: f32,
) -> Element<'a, Message> {
    let count = tabs.len();
    let separators = count.saturating_sub(1) as f32;
    let spare = (available - PLUS_SLOT - separators).max(0.0);
    let tab_width = (spare / count.max(1) as f32)
        .min(max_tab_width)
        .max(MIN_TAB_WIDTH.min(max_tab_width));

    let mut strip: Row<'a, Message> = Row::new().align_y(Alignment::Center).height(Length::Fill);

    for (i, tab) in tabs.iter().enumerate() {
        let is_active = i == active;
        let on_select = tab.on_select.clone();
        let on_close = tab.on_close.clone();

        let close = button(icons::x(14.0).render(false, false))
            .padding(4)
            .on_press_with(move || on_close())
            .style(round_style);

        let content = row![
            icons::folder(16.0).render(false, is_active),
            container(text(tab.title.clone()).size(13).wrapping(Wrapping::None))
                .width(Length::Fill)
                .clip(true),
            close,
        ]
        .spacing(8)
        .align_y(Alignment::Center)
        .height(Length::Fill);

        strip = strip.push(
            button(content)
                .width(Length::Fixed(tab_width))
                .height(Length::Fill)
                .padding(Padding { top: 0.0, right: 6.0, bottom: 0.0, left: 12.0 })
                .on_press_with(move || on_select())
                .style(tab_style(is_active)),
        );

        if i + 1 < count && i != active && i + 1 != active {
            strip = strip.push(
                container(vertical_rule(1))
                    .height(Length::Fill)
                    .padding(Padding { top: 9.0, right: 0.0, bottom: 9.0, left: 0.0 }),
            );
        }
    }

    if let Some(callback) = on_new {
        let callback = callback.clone();
        strip = strip.push(
            container(
                button(icons::plus(16.0).render(false, false))
                    .padding(6)
                    .on_press_with(move || callback())
                    .style(round_style),
            )
            .center_y(Length::Fill)
            .padding(Padding { top: 0.0, right: 0.0, bottom: 0.0, left: 6.0 }),
        );
    }

    strip.into()
}

impl<'a, Message: Clone + 'a> From<TabBar<'a, Message>> for Element<'a, Message> {
    fn from(bar: TabBar<'a, Message>) -> Self {
        let TabBar {
            tabs,
            active,
            max_tab_width,
            on_new,
        } = bar;

        container(responsive(move |size: Size| {
            build_strip(&tabs, active, max_tab_width, on_new.as_ref(), size.width)
        }))
        .width(Length::Fill)
        .height(BAR_HEIGHT)
        .padding(Padding { top: 6.0, right: 8.0, bottom: 0.0, left: 8.0 })
        .style(bar_style)
        .into()
    }
}