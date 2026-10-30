pub mod config;
pub mod drive;
pub mod icons;
pub mod sidebar;
pub mod spitem;
pub mod spmenu;
pub mod tab;
pub mod tabbar;

use std::sync::Arc;

use bootstrap::BootReceiver;
use shared::BootEvent;
use iced::widget::{column, container, row, text};
use iced::{Element, Length, Size, Subscription, Task, Theme, window};
use leoexplorer_setup::{self as setup, SetupState};
use log::debug;
use shared::AllDirectories;

use tab::Tab;
use tabbar::TabBar;

// ------------------------------------------------------------------ main screen (unchanged logic)

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

fn update_main(app: &mut App, msg: Message) -> Task<Message> {
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

fn view_main(app: &App) -> Element<'_, Message> {
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

// ------------------------------------------------------------------ root: setup screen -> main screen
//
// One iced application switches screens, because winit can't create a second
// event loop in the same process on macOS (so two `application().run()` calls
// in a row is not an option).

enum Root {
    Setup(SetupState),
    Main(App),
}

#[derive(Debug, Clone)]
enum RootMessage {
    Boot(BootEvent),
    Setup(setup::Message),
    Main(Message),
}

/// Builds the main screen. Runs only after settings/cache are ready.
fn enter_main() -> Root {
    config::init();

    let dirs = Arc::new(system::get_all_dirs_and_drives());
    debug!("system dirs: {dirs:#?}");

    Root::Main(App::new(dirs))
}

fn update(root: &mut Root, msg: RootMessage) -> Task<RootMessage> {
    match msg {
        RootMessage::Boot(BootEvent::Ready) | RootMessage::Setup(setup::Message::Continue) => {
            if matches!(root, Root::Setup(_)) {
                *root = enter_main();
            }
        }
        RootMessage::Boot(event) => {
            if let Root::Setup(state) = root {
                state.apply(event);
            }
        }
        RootMessage::Setup(m) => {
            if let Root::Setup(state) = root {
                state.update(m);
            }
        }
        RootMessage::Main(m) => {
            if let Root::Main(app) = root {
                return update_main(app, m).map(RootMessage::Main);
            }
        }
    }
    Task::none()
}

fn view(root: &Root) -> Element<'_, RootMessage> {
    match root {
        Root::Setup(state) => state.view().map(RootMessage::Setup),
        Root::Main(app) => view_main(app).map(RootMessage::Main),
    }
}

fn theme(root: &Root) -> Theme {
    match root {
        Root::Setup(_) => Theme::Dark,
        Root::Main(app) => {
            if app.tabs[app.active].sidebar.dark {
                Theme::Dark
            } else {
                Theme::Light
            }
        }
    }
}

fn subscription(root: &Root) -> Subscription<RootMessage> {
    match root {
        Root::Setup(_) => {
            window::frames().map(|at| RootMessage::Setup(setup::Message::Frame(at)))
        }
        Root::Main(_) => Subscription::none(),
    }
}

/// `boot = Some(rx)` starts on the setup screen and follows `rx` until `Ready`.
/// `boot = None` goes straight to the main screen.
pub fn run(boot: Option<BootReceiver>) -> iced::Result {
    let icon = window::icon::from_file_data(
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../assets/app-icon.png")),
        None,
    )
    .ok();

    iced::application("LeoExplorer", update, view)
        .theme(theme)
        .subscription(subscription)
        .window(window::Settings {
            size: Size::new(1100.0, 800.0),
            min_size: Some(Size::new(600.0, 400.0)),
            icon,
            ..Default::default()
        })
        .run_with(move || match boot {
            Some(rx) => (
                Root::Setup(SetupState::new()),
                Task::run(rx, RootMessage::Boot),
            ),
            None => (enter_main(), Task::none()),
        })
}