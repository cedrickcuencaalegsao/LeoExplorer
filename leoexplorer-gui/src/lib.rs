pub mod config;
pub mod drive;
pub mod icons;
pub mod sidebar;
pub mod spitem;
pub mod spmenu;
pub mod tab;
pub mod tabbar;

use std::sync::Arc;

use iced::widget::{column, container, row, text};
use iced::{Element, Length, Size, Task, Theme, window};
use log::debug;
use shared::AllDirectories;

use tab::Tab;
use tabbar::TabBar;

struct App {
    tabs: Vec<Tab>,
    active: usize,
    dirs: Arc<AllDirectories>,
}

impl App {
    fn new(dirs: Arc<AllDirectories>) -> Self {
        Self {
            tabs: vec![Tab::new(false, &dirs)],
            active: 0,
            dirs,
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    Sidebar(usize, sidebar::Message),
    SelectTab(usize),
    CloseTab(usize),
    NewTab,
}

fn update(app: &mut App, msg: Message) -> Task<Message> {
    match msg {
        Message::Sidebar(i, m) => {
            let toggled = matches!(m, sidebar::Message::ToggleTheme);
            if let Some(tab) = app.tabs.get_mut(i) {
                tab.sidebar.update(m);
                if toggled {
                    let dark = tab.sidebar.dark;
                    for other in &mut app.tabs {
                        other.sidebar.dark = dark;
                    }
                }
            }
        }
        Message::SelectTab(i) => {
            if i < app.tabs.len() {
                app.active = i;
            }
        }
        Message::NewTab => {
            let dark = app.tabs[app.active].sidebar.dark;
            app.tabs.push(Tab::new(dark, &app.dirs));
            app.active = app.tabs.len() - 1;
        }
        Message::CloseTab(i) => {
            if app.tabs.len() == 1 {
                return iced::exit();
            }
            if i < app.tabs.len() {
                app.tabs.remove(i);
                if app.active > i {
                    app.active -= 1;
                } else if app.active >= app.tabs.len() {
                    app.active = app.tabs.len() - 1;
                }
            }
        }
    }
    Task::none()
}

fn view(app: &App) -> Element<'_, Message> {
    let mut bar = TabBar::new(app.active).on_new(|| Message::NewTab);

    for (i, tab) in app.tabs.iter().enumerate() {
        bar = bar.tab(
            tab.title(),
            move || Message::SelectTab(i),
            move || Message::CloseTab(i),
        );
    }

    let index = app.active;
    let tab = &app.tabs[index];

    let body = row![
        tab.sidebar.view().map(move |m| Message::Sidebar(index, m)),
        container(text(format!("{} — file list goes here", tab.title())))
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(16),
    ];

    column![bar, body].into()
}

pub fn run() -> iced::Result {
    config::init();

    let dirs = Arc::new(system::get_all_dirs_and_drives());
    debug!("system dirs: {dirs:#?}");

    iced::application("LeoExplorer", update, view)
        .theme(|app| {
            if app.tabs[app.active].sidebar.dark {
                Theme::Dark
            } else {
                Theme::Light
            }
        })
        .window(window::Settings {
            size: Size::new(1100.0, 800.0),
            min_size: Some(Size::new(600.0, 400.0)),
            ..Default::default()
        })
        .run_with(move || (App::new(dirs), Task::none()))
}