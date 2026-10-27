pub mod config;
pub mod icons;
pub mod sidebar;
pub mod spitem;
pub mod spmenu;
pub mod drive;

use iced::widget::{container, row, text};
use iced::{window, Element, Length, Size, Theme};
use sidebar::Sidebar;

#[derive(Default)]
struct App {
    sidebar: Sidebar,
}

#[derive(Debug, Clone)]
enum Message {
    Sidebar(sidebar::Message),
}

fn update(app: &mut App, msg: Message) {
    match msg {
        Message::Sidebar(m) => app.sidebar.update(m),
    }
}

fn view(app: &App) -> Element<'_, Message> {
    row![
        app.sidebar.view().map(Message::Sidebar),
        container(text("File list goes here"))
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(16),
    ]
    .into()
}

pub fn run() -> iced::Result {
    config::init();
    iced::application("LeoExplorer", update, view)
        .theme(|app| if app.sidebar.dark { Theme::Dark } else { Theme::Light })
        .window(window::Settings {
            size: Size::new(1100.0, 800.0),
            min_size: Some(Size::new(600.0, 400.0)),
            ..Default::default()
        })
        .run()
}