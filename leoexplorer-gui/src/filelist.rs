use std::time::SystemTime;

use chrono::{DateTime, Local};
use iced::alignment::Horizontal;
use iced::widget::text::Wrapping;
use iced::widget::{
    Column, Row, button, column, container, responsive, row, scrollable, text, tooltip,
};
use iced::{
    Alignment, Background, Border, Color, Element, Length, Padding, Shadow, Size, Theme, Vector,
};
use shared::{Entry, FileList, RowCallback};

use crate::icons;
use crate::sidebar::{scrollbar_style, thin_scrollbar};
use crate::spitem::fit_label;

const SCROLL_GUTTER: f32 = 12.0;
const CARD_PADDING_X: f32 = 14.0;
const CARD_PADDING_Y: f32 = 10.0;
const COLUMN_SPACING: f32 = 16.0;
const CARD_SPACING: f32 = 8.0;
const ICON_SIZE: f32 = 20.0;
const ICON_GAP: f32 = 10.0;
const LOCK_SIZE: f32 = 14.0;
const NAME_SIZE: f32 = 14.0;
const META_SIZE: f32 = 13.0;
const SAFETY: f32 = 6.0;

const NAME_PORTION: u16 = 4;
const DATE_PORTION: u16 = 3;
const PATH_PORTION: u16 = 5;

const TILE_WIDTH: f32 = 128.0;
const TILE_HEIGHT: f32 = 120.0;
const TILE_PADDING: f32 = 10.0;
const TILE_ICON: f32 = 48.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ViewMode {
    #[default]
    List,
    BigTiles,
}

fn dim(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(Color {
            a: 0.6,
            ..theme.palette().text
        }),
    }
}

fn text_style(theme: &Theme, hidden: bool, dimmed: bool) -> text::Style {
    let alpha = match (hidden, dimmed) {
        (true, _) => 0.4,
        (false, true) => 0.65,
        (false, false) => 1.0,
    };

    text::Style {
        color: Some(Color {
            a: alpha,
            ..theme.palette().text
        }),
    }
}

fn card_style(selected: bool, hidden: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |theme, status| {
        let palette = theme.extended_palette();
        let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);

        let (background, text_color, border_color) = if selected {
            (
                palette.primary.weak.color,
                palette.primary.weak.text,
                palette.primary.base.color,
            )
        } else if hovered {
            (
                palette.background.weak.color,
                palette.background.weak.text,
                palette.background.strong.color,
            )
        } else {
            (
                palette.background.base.color,
                palette.background.base.text,
                palette.background.strong.color,
            )
        };

        button::Style {
            background: Some(Background::Color(background)),
            text_color: if hidden {
                Color {
                    a: 0.5,
                    ..text_color
                }
            } else {
                text_color
            },
            border: Border {
                color: border_color,
                width: 1.0,
                radius: 10.0.into(),
            },
            shadow: Shadow {
                color: Color {
                    a: 0.08,
                    ..Color::BLACK
                },
                offset: Vector::new(0.0, 1.0),
                blur_radius: 3.0,
            },
            ..button::Style::default()
        }
    }
}

fn format_date(time: SystemTime) -> String {
    let local: DateTime<Local> = time.into();
    local.format("%b %-d, %Y  %-I:%M %p").to_string()
}

fn text_cell<'a, Message: 'a>(
    label: &str,
    size: f32,
    width: f32,
    hidden: bool,
    dimmed: bool,
) -> Element<'a, Message> {
    let (shown, truncated) = fit_label(label, size, width);

    let content = container(
        text(shown)
            .size(size)
            .wrapping(Wrapping::None)
            .style(move |theme: &Theme| text_style(theme, hidden, dimmed)),
    )
    .width(Length::Fill)
    .clip(true);

    if truncated {
        tooltip(
            content,
            container(text(label.to_string()).size(13))
                .padding(6)
                .style(container::rounded_box),
            tooltip::Position::FollowCursor,
        )
        .into()
    } else {
        content.into()
    }
}

/// One row in the "list tiles" layout.
fn card<'a, Message: Clone + 'a>(
    entry: &Entry,
    index: usize,
    selected: bool,
    card_width: f32,
    on_click: Option<&RowCallback<'a, Message>>,
) -> Element<'a, Message> {
    let hidden = entry.hidden;

    let lock_space = if entry.permission_required {
        COLUMN_SPACING + LOCK_SIZE
    } else {
        0.0
    };
    let total_portion = (NAME_PORTION + DATE_PORTION + PATH_PORTION) as f32;
    let available =
        (card_width - 2.0 * CARD_PADDING_X - 2.0 * COLUMN_SPACING - lock_space - SAFETY).max(0.0);

    let name_width = available * NAME_PORTION as f32 / total_portion;
    let date_width = available * DATE_PORTION as f32 / total_portion;
    let path_width = available * PATH_PORTION as f32 / total_portion;

    let icon = if entry.is_dir {
        icons::folder(ICON_SIZE)
    } else {
        icons::file_text(ICON_SIZE)
    };

    let date = entry.modified.map(format_date).unwrap_or_default();
    let path = entry.path.display().to_string();

    let name_cell = row![
        icon.render(hidden, false),
        text_cell(
            &entry.name,
            NAME_SIZE,
            name_width - ICON_SIZE - ICON_GAP,
            hidden,
            false
        ),
    ]
    .spacing(ICON_GAP)
    .align_y(Alignment::Center);

    let mut content = row![
        container(name_cell).width(Length::FillPortion(NAME_PORTION)),
        container(text_cell(&date, META_SIZE, date_width, hidden, true))
            .width(Length::FillPortion(DATE_PORTION)),
        container(text_cell(&path, META_SIZE, path_width, hidden, true))
            .width(Length::FillPortion(PATH_PORTION)),
    ]
    .spacing(COLUMN_SPACING)
    .align_y(Alignment::Center);

    if entry.permission_required {
        content = content.push(icons::lock(LOCK_SIZE).render(hidden, false));
    }

    let mut item = button(content)
        .width(Length::Fill)
        .padding(Padding {
            top: CARD_PADDING_Y,
            right: CARD_PADDING_X,
            bottom: CARD_PADDING_Y,
            left: CARD_PADDING_X,
        })
        .style(card_style(selected, hidden));

    if let Some(callback) = on_click {
        let callback = callback.clone();
        item = item.on_press_with(move || callback(index));
    }

    item.into()
}

/// One square tile in the "big tiles" layout.
fn tile<'a, Message: Clone + 'a>(
    entry: &Entry,
    index: usize,
    selected: bool,
    on_click: Option<&RowCallback<'a, Message>>,
) -> Element<'a, Message> {
    let hidden = entry.hidden;

    let icon = if entry.is_dir {
        icons::folder(TILE_ICON)
    } else {
        icons::file_text(TILE_ICON)
    };

    let (shown, truncated) = fit_label(&entry.name, NAME_SIZE, TILE_WIDTH - 2.0 * TILE_PADDING);

    let label = text(shown)
        .size(NAME_SIZE)
        .wrapping(Wrapping::None)
        .width(Length::Fill)
        .align_x(Horizontal::Center)
        .style(move |theme: &Theme| text_style(theme, hidden, false));

    let mut item = button(
        column![icon.render(hidden, false), label]
            .spacing(8)
            .align_x(Horizontal::Center),
    )
    .width(Length::Fixed(TILE_WIDTH))
    .height(Length::Fixed(TILE_HEIGHT))
    .padding(TILE_PADDING)
    .style(card_style(selected, hidden));

    if let Some(callback) = on_click {
        let callback = callback.clone();
        item = item.on_press_with(move || callback(index));
    }

    if truncated {
        tooltip(
            item,
            container(text(entry.name.clone()).size(13))
                .padding(6)
                .style(container::rounded_box),
            tooltip::Position::FollowCursor,
        )
        .into()
    } else {
        item.into()
    }
}

pub fn file_list<'a, Message: Clone + 'a>(
    list: FileList<'a, Message>,
    mode: ViewMode,
) -> Element<'a, Message> {
    let FileList {
        entries,
        selected,
        on_click,
    } = list;

    if entries.is_empty() {
        return container(text("This folder is empty").size(14).style(dim))
            .padding(8)
            .into();
    }

    responsive(move |size: Size| {
        let card_width = (size.width - SCROLL_GUTTER).max(0.0);

        let body: Element<'a, Message> = match mode {
            ViewMode::List => {
                let mut rows = Column::new().spacing(CARD_SPACING);

                for (index, entry) in entries.iter().enumerate() {
                    rows = rows.push(card(
                        entry,
                        index,
                        selected == Some(index),
                        card_width,
                        on_click.as_ref(),
                    ));
                }

                rows.into()
            }
            ViewMode::BigTiles => {
                let per_row =
                    (((card_width + CARD_SPACING) / (TILE_WIDTH + CARD_SPACING)) as usize).max(1);

                let mut grid = Column::new().spacing(CARD_SPACING);

                for (chunk_index, chunk) in entries.chunks(per_row).enumerate() {
                    let mut line = Row::new().spacing(CARD_SPACING);

                    for (offset, entry) in chunk.iter().enumerate() {
                        let index = chunk_index * per_row + offset;
                        line = line.push(tile(
                            entry,
                            index,
                            selected == Some(index),
                            on_click.as_ref(),
                        ));
                    }

                    grid = grid.push(line);
                }

                grid.into()
            }
        };

        scrollable(container(body).padding(Padding {
            right: SCROLL_GUTTER,
            ..Padding::ZERO
        }))
        .width(Length::Fill)
        .height(Length::Fill)
        .direction(thin_scrollbar())
        .style(scrollbar_style)
        .into()
    })
    .into()
}

pub trait FileListView<'a, Message> {
    fn view(self, mode: ViewMode) -> Element<'a, Message>;
}

impl<'a, Message: Clone + 'a> FileListView<'a, Message> for FileList<'a, Message> {
    fn view(self, mode: ViewMode) -> Element<'a, Message> {
        file_list(self, mode)
    }
}