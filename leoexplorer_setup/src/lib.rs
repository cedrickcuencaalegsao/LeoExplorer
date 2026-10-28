//! The first-run "getting ready" screen: a centered card with the app icon,
//! a step list and an animated progress bar. Pure iced widgets, no canvas.

use std::f32::consts::TAU;
use std::time::Instant;

use bootstrap::{BootEvent, Step};
use iced::font::Weight;
use iced::widget::image::Handle;
use iced::widget::text::Wrapping;
use iced::widget::{button, column, container, image, progress_bar, row, text, Column, Space};
use iced::{
    gradient, Alignment, Background, Border, Color, Element, Font, Gradient, Length, Radians,
    Shadow, Theme, Vector,
};

const STEPS: [(Step, &str); 2] = [
    (Step::Settings, "Preparing settings"),
    (Step::Cache, "Building file cache"),
];

const CARD_WIDTH: f32 = 420.0;

#[derive(Debug, Clone)]
pub enum Message {
    /// Animation tick.
    Frame(Instant),
    /// Shown after a failure: open the app anyway.
    Continue,
}

pub struct SetupState {
    step: Step,
    detail: String,
    progress: Option<f32>,
    error: Option<String>,
    icon: Handle,
    started: Instant,
    now: Instant,
}

impl Default for SetupState {
    fn default() -> Self {
        Self::new()
    }
}

impl SetupState {
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            step: Step::Settings,
            detail: String::new(),
            progress: None,
            error: None,
            icon: Handle::from_bytes(
                include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../assets/app-icon.png"))
                    .as_slice(),
            ),
            started: now,
            now,
        }
    }

    /// Feed a progress event from `bootstrap`.
    pub fn apply(&mut self, event: BootEvent) {
        match event {
            BootEvent::Step(step) => {
                self.step = step;
                self.detail.clear();
            }
            BootEvent::Detail(detail) => self.detail = detail,
            BootEvent::Progress(p) => self.progress = Some(p.clamp(0.0, 1.0)),
            BootEvent::Failed(error) => self.error = Some(error),
            BootEvent::Ready => {}
        }
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::Frame(now) => self.now = now,
            Message::Continue => {} // handled by the parent
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let elapsed = self.now.saturating_duration_since(self.started).as_secs_f32();
        let pulse = 0.5 + 0.5 * (elapsed * TAU / 1.2).sin();

        // ---- header
        let header = column![
            image(self.icon.clone()).width(64).height(64),
            column![
                text("Leo Explorer").size(26).font(bold()),
                text("Getting things ready for your first launch")
                    .size(14)
                    .style(dim),
            ]
            .spacing(6)
            .align_x(Alignment::Center),
        ]
        .spacing(18)
        .align_x(Alignment::Center);

        // ---- steps
        let mut steps = Column::new().spacing(14);
        for (step, label) in STEPS {
            steps = steps.push(self.step_row(step, label, pulse));
        }

        // ---- progress bar + status line
        let detail = if self.detail.is_empty() {
            "Getting started…".to_string()
        } else {
            self.detail.clone()
        };

        let mut status_line = row![container(
            text(detail).size(12).style(dim).wrapping(Wrapping::None)
        )
        .width(Length::Fill)
        .clip(true)]
        .spacing(8)
        .align_y(Alignment::Center);

        if let Some(p) = self.progress {
            status_line = status_line.push(text(format!("{:.0}%", p * 100.0)).size(12).style(dim));
        }

        let progress = column![self.bar(elapsed), status_line].spacing(8);

        // ---- card
        let mut content = Column::new()
            .spacing(26)
            .width(CARD_WIDTH)
            .align_x(Alignment::Center)
            .push(header)
            .push(steps)
            .push(progress);

        if let Some(error) = &self.error {
            content = content.push(
                container(
                    column![
                        text(error.clone()).size(13).style(text::danger),
                        button(text("Continue anyway").size(13))
                            .padding([6, 14])
                            .style(button::primary)
                            .on_press(Message::Continue),
                    ]
                    .spacing(10),
                )
                .width(Length::Fill)
                .padding(12)
                .style(error_style),
            );
        } else {
            content = content.push(text("This only happens once.").size(12).style(dim));
        }

        let card = container(content).padding(32).style(card_style);

        container(card)
            .center(Length::Fill)
            .padding(24)
            .style(page_style)
            .into()
    }

    fn step_row(&self, step: Step, label: &'static str, pulse: f32) -> Element<'_, Message> {
        let failed = self.error.is_some() && step == self.step;
        let active = step == self.step && self.error.is_none();
        let done = step < self.step;

        let (mark, status) = if failed {
            (Mark::Failed, "Failed")
        } else if done {
            (Mark::Done, "Done")
        } else if active {
            (Mark::Active(pulse), "In progress")
        } else {
            (Mark::Pending, "Waiting")
        };

        let pending = matches!(mark, Mark::Pending);

        row![
            indicator(mark),
            text(label)
                .size(15)
                .width(Length::Fill)
                .style(move |theme: &Theme| {
                    if pending {
                        dim(theme)
                    } else {
                        text::Style { color: None }
                    }
                }),
            text(status).size(12).style(move |theme: &Theme| {
                let palette = theme.extended_palette();
                let color = if failed {
                    palette.danger.base.color
                } else if active {
                    palette.primary.base.color
                } else {
                    Color { a: 0.55, ..theme.palette().text }
                };
                text::Style { color: Some(color) }
            }),
        ]
        .spacing(12)
        .align_y(Alignment::Center)
        .into()
    }

    /// Real bar when a percentage is known, otherwise a sliding highlight.
    fn bar(&self, elapsed: f32) -> Element<'_, Message> {
        if let Some(p) = self.progress {
            return progress_bar(0.0..=1.0, p)
                .height(6)
                .style(|theme: &Theme| {
                    let palette = theme.extended_palette();
                    progress_bar::Style {
                        background: Background::Color(track_color(theme)),
                        bar: Background::Color(palette.primary.base.color),
                        border: Border {
                            radius: 3.0.into(),
                            ..Border::default()
                        },
                    }
                })
                .into();
        }

        // Indeterminate: a highlight that slides left to right, built from
        // FillPortion spacers (1000 units wide).
        let t = (elapsed % 1.6) / 1.6;
        let chunk = 320.0;
        let x = t * (1000.0 + chunk) - chunk;
        let start = x.max(0.0);
        let end = (x + chunk).min(1000.0);

        let left = (start as u16).max(1);
        let mid = ((end - start) as u16).max(1);
        let right = ((1000.0 - end) as u16).max(1);

        let highlight = row![
            Space::with_width(Length::FillPortion(left)),
            container(Space::new(Length::Fill, Length::Fill))
                .width(Length::FillPortion(mid))
                .height(Length::Fill)
                .style(|theme: &Theme| container::Style {
                    background: Some(Background::Color(theme.extended_palette().primary.base.color)),
                    border: Border {
                        radius: 3.0.into(),
                        ..Border::default()
                    },
                    ..container::Style::default()
                }),
            Space::with_width(Length::FillPortion(right)),
        ]
        .height(Length::Fill);

        container(highlight)
            .width(Length::Fill)
            .height(6)
            .clip(true)
            .style(|theme: &Theme| container::Style {
                background: Some(Background::Color(track_color(theme))),
                border: Border {
                    radius: 3.0.into(),
                    ..Border::default()
                },
                ..container::Style::default()
            })
            .into()
    }
}

// ---------------------------------------------------------------- step indicator

#[derive(Clone, Copy)]
enum Mark {
    Done,
    Active(f32),
    Pending,
    Failed,
}

fn indicator(mark: Mark) -> Element<'static, Message> {
    let inner: Element<'static, Message> = match mark {
        Mark::Done => text("✓")
            .size(13)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.extended_palette().primary.base.text),
            })
            .into(),
        Mark::Failed => text("!")
            .size(13)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.extended_palette().danger.base.text),
            })
            .into(),
        Mark::Active(pulse) => container(Space::new(8, 8))
            .style(move |theme: &Theme| container::Style {
                background: Some(Background::Color(Color {
                    a: 0.45 + 0.55 * pulse,
                    ..theme.extended_palette().primary.base.color
                })),
                border: Border {
                    radius: 4.0.into(),
                    ..Border::default()
                },
                ..container::Style::default()
            })
            .into(),
        Mark::Pending => Space::new(0, 0).into(),
    };

    container(inner)
        .center(24)
        .style(move |theme: &Theme| {
            let palette = theme.extended_palette();
            let (fill, ring) = match mark {
                Mark::Done => (Some(palette.primary.base.color), palette.primary.base.color),
                Mark::Failed => (Some(palette.danger.base.color), palette.danger.base.color),
                Mark::Active(pulse) => (
                    Some(Color {
                        a: 0.08 + 0.14 * pulse,
                        ..palette.primary.base.color
                    }),
                    palette.primary.base.color,
                ),
                Mark::Pending => (None, Color { a: 0.6, ..palette.background.strong.color }),
            };

            container::Style {
                background: fill.map(Background::Color),
                border: Border {
                    color: ring,
                    width: 1.5,
                    radius: 12.0.into(),
                },
                ..container::Style::default()
            }
        })
        .into()
}

// ---------------------------------------------------------------- styles

fn bold() -> Font {
    Font {
        weight: Weight::Bold,
        ..Font::DEFAULT
    }
}

fn dim(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(Color { a: 0.6, ..theme.palette().text }),
    }
}

fn track_color(theme: &Theme) -> Color {
    Color {
        a: 0.55,
        ..theme.extended_palette().background.strong.color
    }
}

fn mix(a: Color, b: Color, t: f32) -> Color {
    Color {
        r: a.r + (b.r - a.r) * t,
        g: a.g + (b.g - a.g) * t,
        b: a.b + (b.b - a.b) * t,
        a: 1.0,
    }
}

/// Window background: a soft diagonal gradient tinted with the accent color.
fn page_style(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();
    let base = palette.background.base.color;
    let tint = mix(base, palette.primary.base.color, 0.16);

    container::Style {
        background: Some(Background::Gradient(Gradient::Linear(
            gradient::Linear::new(Radians(2.4))
                .add_stop(0.0, base)
                .add_stop(1.0, tint),
        ))),
        ..container::Style::default()
    }
}

fn card_style(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();

    container::Style {
        background: Some(Background::Color(palette.background.weak.color)),
        border: Border {
            color: Color { a: 0.5, ..palette.background.strong.color },
            width: 1.0,
            radius: 18.0.into(),
        },
        shadow: Shadow {
            color: Color { a: 0.35, ..Color::BLACK },
            offset: Vector::new(0.0, 12.0),
            blur_radius: 32.0,
        },
        ..container::Style::default()
    }
}

fn error_style(theme: &Theme) -> container::Style {
    let danger = theme.extended_palette().danger.base.color;

    container::Style {
        background: Some(Background::Color(Color { a: 0.12, ..danger })),
        border: Border {
            color: Color { a: 0.5, ..danger },
            width: 1.0,
            radius: 10.0.into(),
        },
        ..container::Style::default()
    }
}