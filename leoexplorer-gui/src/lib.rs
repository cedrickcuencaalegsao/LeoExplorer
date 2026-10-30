pub mod config;
pub mod drive;
pub mod filelist;
pub mod icons;
pub mod sidebar;
pub mod spitem;
pub mod spmenu;
pub mod tab;
pub mod tabbar;
pub mod titlebar;
pub mod item_menu;

use std::sync::Arc;

use shared::BootEvent;
use iced::futures::channel::mpsc::UnboundedReceiver;
use iced::widget::{column, container, text, Column};
use iced::{Alignment, Element, Length, Size, Task, Theme, window};
use log::debug;
use shared::AllDirectories;

use tab::Tab;
use tabbar::TabBar;

const MAX_SETUP_LINES: usize = 8;

struct Workspace {
    tabs: Vec<Tab>,
    active: usize,
    dirs: Arc<AllDirectories>,
}

enum Screen {
    Setup(Vec<String>),
    Main(Workspace),
}

struct App {
    screen: Screen,
}

#[derive(Debug, Clone)]
enum Message {
    Boot(BootEvent),
    BootFinished,
    Tab(usize, tab::Message),
    SelectTab(usize),
    CloseTab(usize),
    NewTab,
    SearchChanged(String),
    SearchSubmitted,
    DragWindow,
}

impl Workspace {
    fn new() -> Self {
        let dirs = Arc::new(system::get_all_dirs_and_drives());
        debug!("system dirs: {dirs:#?}");

        Self {
            tabs: vec![Tab::new(false, &dirs)],
            active: 0,
            dirs,
        }
    }

    fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::Tab(i, m) => {
                let toggled_theme =
                    matches!(m, tab::Message::Sidebar(sidebar::Message::ToggleTheme));

                if let Some(tab) = self.tabs.get_mut(i) {
                    tab.update(m);

                    if toggled_theme {
                        let dark = tab.sidebar.dark;
                        for other in &mut self.tabs {
                            other.sidebar.dark = dark;
                        }
                    }
                }

                Task::none()
            }
            Message::SelectTab(i) => {
                if i < self.tabs.len() {
                    self.active = i;
                }
                Task::none()
            }
            Message::NewTab => {
                let dark = self.tabs[self.active].sidebar.dark;
                self.tabs.push(Tab::new(dark, &self.dirs));
                self.active = self.tabs.len() - 1;
                Task::none()
            }
            Message::CloseTab(i) => {
                if self.tabs.len() == 1 {
                    return iced::exit();
                }
                if i < self.tabs.len() {
                    self.tabs.remove(i);
                    if self.active > i {
                        self.active -= 1;
                    } else if self.active >= self.tabs.len() {
                        self.active = self.tabs.len() - 1;
                    }
                }
                Task::none()
            }
            Message::SearchChanged(text) => {
                // Live filter: forward to the active tab.
                if let Some(tab) = self.tabs.get_mut(self.active) {
                    tab.update(tab::Message::Search(text));
                }
                Task::none()
            }
            Message::SearchSubmitted => Task::none(),
            // The window drag message is handled in the top-level `update`.
            Message::Boot(_) | Message::BootFinished | Message::DragWindow => Task::none(),
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let mut bar = TabBar::new(self.active).on_new(|| Message::NewTab);

        for (i, tab) in self.tabs.iter().enumerate() {
            bar = bar.tab(
                tab.title(),
                move || Message::SelectTab(i),
                move || Message::CloseTab(i),
            );
        }

        let index = self.active;
        let body = self.tabs[index].view().map(move |m| Message::Tab(index, m));

        column![bar, body].into()
    }
}

fn setup_view(lines: &[String]) -> Element<'_, Message> {
    let mut log = Column::new().spacing(4);
    for line in lines {
        log = log.push(text(line.as_str()).size(13));
    }

    container(
        column![
            text("Setting up LeoExplorer").size(28),
            text("Building settings and the file cache. This only happens on the first run.")
                .size(14),
            log,
        ]
        .spacing(16)
        .align_x(Alignment::Center),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .into()
}

fn update(app: &mut App, msg: Message) -> Task<Message> {
    match msg {
        Message::Boot(event) => {
            if let Screen::Setup(lines) = &mut app.screen {
                lines.push(format!("{event:?}"));
                if lines.len() > MAX_SETUP_LINES {
                    lines.remove(0);
                }
            }
            Task::none()
        }
        Message::BootFinished => {
            if matches!(app.screen, Screen::Setup(_)) {
                app.screen = Screen::Main(Workspace::new());
            }
            Task::none()
        }
        Message::DragWindow => window::get_oldest().and_then(window::drag),
        other => match &mut app.screen {
            Screen::Main(workspace) => workspace.update(other),
            Screen::Setup(_) => Task::none(),
        },
    }
}

fn view(app: &App) -> Element<'_, Message> {
    let (search, content) = match &app.screen {
        Screen::Setup(lines) => (String::new(), setup_view(lines)),
        Screen::Main(workspace) => (
            workspace.tabs[workspace.active].search().to_string(),
            workspace.view(),
        ),
    };

    let title_bar = titlebar::view(
        &search,
        Message::SearchChanged,
        Message::SearchSubmitted,
        Message::DragWindow,
    );

    column![title_bar, content].into()
}

fn theme(app: &App) -> Theme {
    match &app.screen {
        Screen::Main(workspace) if workspace.tabs[workspace.active].sidebar.dark => Theme::Dark,
        _ => Theme::Light,
    }
}

fn window_settings() -> window::Settings {
    let base = window::Settings {
        size: Size::new(1100.0, 800.0),
        min_size: Some(Size::new(600.0, 400.0)),
        ..Default::default()
    };

    // macOS: keep the native traffic lights, but let our bar draw
    // underneath the title bar area (titlebar.rs leaves room for them).
    #[cfg(target_os = "macos")]
    {
        window::Settings {
            platform_specific: window::settings::PlatformSpecific {
                title_hidden: true,
                titlebar_transparent: true,
                fullsize_content_view: true,
            },
            ..base
        }
    }

    // Windows / Linux: the normal OS title bar stays; our bar sits below it.
    #[cfg(not(target_os = "macos"))]
    {
        base
    }
}

pub fn run(boot: Option<UnboundedReceiver<BootEvent>>) -> iced::Result {
    config::init();

    iced::application("Leo Explorer", update, view)
        .theme(theme)
        .window(window_settings())
        .run_with(move || match boot {
            Some(receiver) => (
                App {
                    screen: Screen::Setup(Vec::new()),
                },
                Task::run(receiver, Message::Boot).chain(Task::done(Message::BootFinished)),
            ),
            None => (
                App {
                    screen: Screen::Main(Workspace::new()),
                },
                Task::none(),
            ),
        })
}