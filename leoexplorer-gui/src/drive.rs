use iced::widget::{button, column, horizontal_space, progress_bar, row, text};
use iced::{Alignment, Background, Border, Element, Length, Theme};

use crate::icons;

pub struct Drive {
    pub label: String,
    pub used_gb: f32,
    pub total_gb: f32,
    pub removable: bool,
}

impl Drive {
    pub fn new(label: impl Into<String>, used_gb: f32, total_gb: f32) -> Self {
        Self {
            label: label.into(),
            used_gb,
            total_gb,
            removable: false,
        }
    }

    pub fn removable(mut self) -> Self {
        self.removable = true;
        self
    }

    pub fn free_gb(&self) -> f32 {
        (self.total_gb - self.used_gb).max(0.0)
    }

    pub fn used_ratio(&self) -> f32 {
        if self.total_gb <= 0.0 {
            0.0
        } else {
            (self.used_gb / self.total_gb).clamp(0.0, 1.0)
        }
    }

    pub fn view<'a, Message: Clone + 'a>(
        &'a self,
        on_eject: impl Fn() -> Message + 'a,
    ) -> Element<'a, Message> {
        let drive_icon = if self.removable {
            icons::usb(28.0)
        } else {
            icons::hard_drive(28.0)
        };

        let mut title = row![text(&self.label).size(14), horizontal_space()]
            .align_y(Alignment::Center);

        if self.removable {
            title = title.push(
                button(icons::eject(14.0))
                    .on_press_with(on_eject)
                    .style(button::text),
            );
        }

        let details = column![
            title,
            progress_bar(0.0..=1.0, self.used_ratio())
                .height(6)
                .style(bar_style),
            text(format!(
                "free {:.0} GB · used {:.0} GB of {:.0} GB",
                self.free_gb(),
                self.used_gb,
                self.total_gb
            ))
            .size(11),
        ]
        .spacing(4)
        .width(Length::Fill);

        row![drive_icon, details]
            .spacing(12)
            .align_y(Alignment::Center)
            .into()
    }
}

fn bar_style(theme: &Theme) -> progress_bar::Style {
    let palette = theme.extended_palette();
    progress_bar::Style {
        background: Background::Color(palette.background.strong.color),
        bar: Background::Color(theme.palette().primary),
        border: Border { radius: 3.0.into(), ..Border::default() },
    }
}