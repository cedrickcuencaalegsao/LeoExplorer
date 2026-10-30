use std::rc::Rc;

use iced::widget::text::Wrapping;
use iced::widget::{
    column, container, mouse_area, responsive, row, text, text_input, Space,
};
use iced::{Alignment, Background, Element, Length, Size, Theme};

pub const HEIGHT: f32 = 36.0;
const TITLE: &str = "Leo Explorer";
const IS_MAC: bool = cfg!(target_os = "macos");

const SEARCH_MAX_WIDTH: f32 = 420.0;
const SEARCH_MARGIN: f32 = 10.0;
/// Empty strip above the search box that still drags the window.
const SEARCH_TOP_GAP: f32 = 6.0;

/// macOS only: room reserved for the native traffic-light buttons.
const TRAFFIC_LIGHTS: f32 = 100.0;

/// Width of the left and right slots. They are equal so the search box is
/// truly centered. Only macOS needs them (traffic lights + title on the
/// left); on Windows and Linux the OS title bar already shows the buttons
/// and the title, so the slots are empty.
const SLOT: f32 = if IS_MAC { 176.0 } else { 0.0 };

fn bar_style(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();
    container::Style {
        background: Some(Background::Color(palette.background.weak.color)),
        ..container::Style::default()
    }
}

fn pill_style(theme: &Theme, status: text_input::Status) -> text_input::Style {
    let palette = theme.extended_palette();
    let mut style = text_input::default(theme, status);

    style.background = Background::Color(palette.background.base.color);
    style.border.radius = 8.0.into();
    style.border.width = 1.0;
    style.border.color = if matches!(status, text_input::Status::Focused) {
        palette.primary.base.color
    } else {
        palette.background.strong.color
    };

    style
}

/// An empty area that drags the window when pressed.
fn drag_zone<'a, Message: Clone + 'a>(
    width: Length,
    height: Length,
    on_drag: Message,
) -> Element<'a, Message> {
    mouse_area(Space::new(width, height))
        .on_press(on_drag)
        .into()
}

/// Bar with a centered search box (like VS Code's command center).
///
/// The native window buttons are used on every platform: on macOS the
/// traffic lights sit in the left slot of this bar, on Windows and Linux
/// the OS title bar sits above it. Only the empty areas around the search
/// box drag the window, the search box itself does not.
pub fn view<'a, Message: Clone + 'a>(
    search: &str,
    on_input: impl Fn(String) -> Message + 'a,
    on_submit: Message,
    on_drag: Message,
) -> Element<'a, Message> {
    let search = search.to_string();
    let on_input: Rc<dyn Fn(String) -> Message + 'a> = Rc::new(on_input);

    let middle_drag = on_drag.clone();
    let middle = responsive(move |size: Size| {
        let pill_width = (size.width - SEARCH_MARGIN).clamp(0.0, SEARCH_MAX_WIDTH);
        let on_input = on_input.clone();

        let input = text_input("Search files and folders", &search)
            .on_input(move |s| on_input(s))
            .on_submit(on_submit.clone())
            .padding([5, 12])
            .size(13)
            .width(Length::Fixed(pill_width))
            .style(pill_style);

        let pill = column![
            drag_zone(
                Length::Fixed(pill_width),
                Length::Fixed(SEARCH_TOP_GAP),
                middle_drag.clone()
            ),
            input,
            drag_zone(Length::Fixed(pill_width), Length::Fill, middle_drag.clone()),
        ]
        .height(Length::Fill);

        row![
            drag_zone(Length::Fill, Length::Fill, middle_drag.clone()),
            pill,
            drag_zone(Length::Fill, Length::Fill, middle_drag.clone()),
        ]
        .height(Length::Fill)
        .into()
    });

    let left: Element<'a, Message> = if IS_MAC {
        // Empty space for the native traffic lights, then the app title.
        let title = container(text(TITLE).size(13).wrapping(Wrapping::None))
            .center_y(Length::Fill)
            .width(Length::Fill)
            .clip(true);

        container(
            row![
                drag_zone(Length::Fixed(TRAFFIC_LIGHTS), Length::Fill, on_drag.clone()),
                mouse_area(title).on_press(on_drag.clone()),
            ]
            .align_y(Alignment::Center)
            .height(Length::Fill),
        )
        .width(Length::Fixed(SLOT))
        .height(Length::Fill)
        .into()
    } else {
        drag_zone(Length::Fixed(SLOT), Length::Fill, on_drag.clone())
    };

    let right = drag_zone(Length::Fixed(SLOT), Length::Fill, on_drag);

    container(
        row![left, middle, right]
            .align_y(Alignment::Start)
            .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(HEIGHT)
    .style(bar_style)
    .into()
}