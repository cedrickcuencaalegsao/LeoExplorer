use std::path::{Component, Path, PathBuf};
use std::rc::Rc;
use std::time::{Duration, Instant};

use iced::widget::text::Wrapping;
use iced::widget::{
    Column, Row, Space, button, column, container, mouse_area, row, scrollable, stack, text,
    tooltip,
};
use iced::{Alignment, Element, Length, Point};
use shared::{AllDirectories, Entry, FileList, ViewMode};

use crate::filelist::FileListView;
use crate::icons;
use crate::item_menu::{ItemAction, ItemMenu, MenuCallback, menu_content};
use crate::sidebar::{self, Sidebar};

const DOUBLE_CLICK: Duration = Duration::from_millis(400);
const MAX_CRUMBS: usize = 5;
const HIDDEN_KEY: usize = usize::MAX;

#[derive(Debug, Clone)]
pub enum Message {
    Sidebar(sidebar::Message),
    ListClicked(usize),
    GoUp,
    NavigateTo(PathBuf),
    SetViewMode(ViewMode),
    /// Chevron after crumb `index`: show the subfolders of `path`.
    ToggleChildren(usize, PathBuf),
    /// "…" button: show the collapsed parent folders.
    ToggleHidden,
    /// Header eye button: show or hide hidden files and folders.
    ToggleShowHidden,
    /// Search box text changed: filter the current folder by name.
    Search(String),
    /// An item was right-clicked: open its popup menu.
    OpenMenu(usize),
    /// An action was chosen in an item's popup menu.
    MenuAction(usize, ItemAction),
    /// The popup menu was dismissed (click outside it).
    CloseMenu,
    /// The mouse moved inside the main pane (position relative to the pane).
    CursorMoved(Point),
}

/// Dropdown shown under the breadcrumb bar.
struct CrumbMenu {
    key: usize,
    items: Vec<(String, PathBuf)>,
}

pub struct Tab {
    pub sidebar: Sidebar,
    title: String,
    current_dir: Option<PathBuf>,
    /// Everything in the folder (after the hidden-files filter).
    all_entries: Vec<Entry>,
    /// What is actually shown: `all_entries` filtered by `search`.
    entries: Vec<Entry>,
    search: String,
    list_selected: Option<usize>,
    last_click: Option<(usize, Instant)>,
    status: Option<String>,
    view_mode: ViewMode,
    crumb_menu: Option<CrumbMenu>,
    show_hidden: bool,
    /// Index (into `entries`) of the item whose popup menu is open.
    menu_open: Option<usize>,
    /// Last known mouse position inside the main pane.
    cursor: Point,
    /// Where the open popup menu is drawn (cursor position at right-click).
    menu_pos: Point,
}

fn filter_hidden(mut entries: Vec<Entry>, show_hidden: bool) -> Vec<Entry> {
    if !show_hidden {
        entries.retain(|entry| !entry.hidden);
    }
    entries
}

fn display_name(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| path.display().to_string())
}

/// Every ancestor of `dir` (including `dir`) as `(label, full path)`.
fn path_crumbs(dir: &Path) -> Vec<(String, PathBuf)> {
    let mut acc = PathBuf::new();

    dir.components()
        .map(|component| {
            acc.push(component.as_os_str());
            let label = match component {
                Component::RootDir => "/".to_string(),
                other => other.as_os_str().to_string_lossy().into_owned(),
            };
            (label, acc.clone())
        })
        .collect()
}

fn mode_button<'a>(
    icon: impl Into<Element<'a, Message>>,
    mode: ViewMode,
    active: bool,
) -> Element<'a, Message> {
    button(icon)
        .padding(6)
        .on_press(Message::SetViewMode(mode))
        .style(move |theme, status| {
            if active {
                button::primary(theme, status)
            } else {
                button::text(theme, status)
            }
        })
        .into()
}

impl Tab {
    pub fn new(dark: bool, dirs: &AllDirectories) -> Self {
        let mut sidebar = Sidebar::from_system(dirs);
        sidebar.dark = dark;

        let mut tab = Self {
            sidebar,
            title: "New Tab".to_string(),
            current_dir: None,
            all_entries: Vec::new(),
            entries: Vec::new(),
            search: String::new(),
            list_selected: None,
            last_click: None,
            status: None,
            view_mode: ViewMode::default(),
            crumb_menu: None,
            show_hidden: false,
            menu_open: None,
            cursor: Point::ORIGIN,
            menu_pos: Point::ORIGIN,
        };

        if let Some(first) = tab.sidebar.section_path(0) {
            tab.navigate(first);
        }

        tab
    }

    pub fn title(&self) -> String {
        self.title.clone()
    }

    /// Current text of this tab's search box.
    pub fn search(&self) -> &str {
        &self.search
    }

    /// Rebuild `entries` from `all_entries` using the current search text.
    fn apply_search(&mut self) {
        let query = self.search.trim().to_lowercase();

        self.entries = if query.is_empty() {
            self.all_entries.clone()
        } else {
            self.all_entries
                .iter()
                .filter(|entry| entry.name.to_lowercase().contains(&query))
                .cloned()
                .collect()
        };

        self.list_selected = None;
        self.last_click = None;
        // Indices changed, so any open item menu would point at the wrong item.
        self.menu_open = None;
    }

    fn navigate(&mut self, path: PathBuf) {
        self.status = None;
        self.last_click = None;
        self.list_selected = None;
        self.crumb_menu = None;
        self.menu_open = None;
        self.search.clear();

        let show = self.show_hidden;
        match sidebar::read_children(&path).map(|entries| filter_hidden(entries, show)) {
            Ok(entries) => self.all_entries = entries,
            Err(err) => {
                self.all_entries.clear();
                self.status = Some(err.to_string());
            }
        }
        self.apply_search();

        self.title = self
            .sidebar
            .sections
            .iter()
            .find(|section| section.path.as_ref() == Some(&path))
            .map(|section| section.name.clone())
            .unwrap_or_else(|| display_name(&path));

        self.current_dir = Some(path);
    }

    pub fn refresh(&mut self) {
        let Some(dir) = self.current_dir.clone() else {
            return;
        };

        if !dir.exists() {
            if let Some(parent) = dir.parent() {
                self.navigate(parent.to_path_buf());
            }
            return;
        }

        let show = self.show_hidden;
        match sidebar::read_children(&dir).map(|entries| filter_hidden(entries, show)) {
            Ok(entries) => self.all_entries = entries,
            Err(err) => {
                log::warn!("cannot read {}: {err}", dir.display());
                self.all_entries.clear();
            }
        }

        // Keeps the current search text applied to the fresh listing.
        self.apply_search();
    }

    /// Runs the action chosen in an item's popup menu.
    fn run_menu_action(&mut self, index: usize, action: ItemAction) {
        let Some(dir) = self.current_dir.clone() else {
            return;
        };
        let Some(entry) = self.entries.get(index) else {
            return;
        };

        let path = dir.join(&entry.name);
        let is_dir = entry.is_dir;

        match action {
            ItemAction::Open => {
                if is_dir {
                    self.navigate(path);
                } else {
                    // TODO: open the file with the system default app
                    // (e.g. with the `open` crate: `open::that(&path)`).
                    self.status = Some(format!("Open is not wired up yet: {}", path.display()));
                }
            }
            ItemAction::Rename => {
                // TODO: start an inline rename / show a rename dialog.
                self.status = Some("Rename is not implemented yet".to_string());
            }
            ItemAction::Copy => {
                // TODO: put `path` on the clipboard (iced::clipboard::write
                // needs a Task, so this has to move to a Task-returning update).
                self.status = Some("Copy is not implemented yet".to_string());
            }
            ItemAction::Delete => {
                // WARNING: permanent delete, no confirmation and no trash.
                let result = if is_dir {
                    std::fs::remove_dir_all(&path)
                } else {
                    std::fs::remove_file(&path)
                };

                match result {
                    Ok(()) => {
                        self.refresh();
                    }
                    Err(err) => {
                        self.status = Some(format!("Cannot delete {}: {err}", path.display()));
                    }
                }
            }
            ItemAction::Properties => {
                self.status = Some(match std::fs::metadata(&path) {
                    Ok(meta) => format!(
                        "{} - {} - {} bytes",
                        path.display(),
                        if is_dir { "folder" } else { "file" },
                        meta.len()
                    ),
                    Err(err) => format!("Cannot read {}: {err}", path.display()),
                });
            }
        }
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::Sidebar(m) => {
                let target = match &m {
                    sidebar::Message::ToggleSection(i) => {
                        self.sidebar.section_path(*i).map(|path| (path, true))
                    }
                    sidebar::Message::OpenEntry(i, j) => self.sidebar.entry_path(*i, *j),
                    _ => None,
                };

                self.sidebar.update(m);

                if let Some((path, true)) = target {
                    self.navigate(path);
                }
            }
            Message::ListClicked(index) => {
                self.crumb_menu = None;
                // A left click anywhere on an item closes any open popup menu.
                self.menu_open = None;

                let Some(dir) = self.current_dir.clone() else {
                    return;
                };
                let Some(entry) = self.entries.get(index) else {
                    return;
                };

                let path = dir.join(&entry.name);
                let is_dir = entry.is_dir;

                let double = matches!(
                    self.last_click,
                    Some((last, at)) if last == index && at.elapsed() < DOUBLE_CLICK
                );

                if double {
                    // Double click: open folders.
                    self.last_click = None;
                    if is_dir {
                        self.navigate(path);
                    }
                } else {
                    // Single click: just select. Right-click opens the menu.
                    self.last_click = Some((index, Instant::now()));
                    self.list_selected = Some(index);
                    self.status = None;
                }
            }
            Message::OpenMenu(index) => {
                if index < self.entries.len() {
                    self.crumb_menu = None;
                    self.last_click = None;
                    self.list_selected = Some(index);
                    self.menu_pos = self.cursor;
                    self.menu_open = Some(index);
                    self.status = None;
                }
            }
            Message::MenuAction(index, action) => {
                self.menu_open = None;
                self.run_menu_action(index, action);
            }
            Message::CloseMenu => {
                self.menu_open = None;
            }
            Message::CursorMoved(point) => {
                // Frozen while the menu is open so the menu doesn't move.
                if self.menu_open.is_none() {
                    self.cursor = point;
                }
            }
            Message::GoUp => {
                let parent = self
                    .current_dir
                    .as_ref()
                    .and_then(|dir| dir.parent())
                    .map(Path::to_path_buf);

                if let Some(parent) = parent {
                    self.navigate(parent);
                }
            }
            Message::NavigateTo(path) => self.navigate(path),
            Message::SetViewMode(mode) => {
                self.crumb_menu = None;
                self.menu_open = None;
                self.view_mode = mode;
            }
            Message::ToggleChildren(index, path) => {
                self.menu_open = None;

                if self.crumb_menu.as_ref().is_some_and(|menu| menu.key == index) {
                    self.crumb_menu = None;
                    return;
                }

                let show = self.show_hidden;
                let mut items: Vec<(String, PathBuf)> = sidebar::read_children(&path)
                    .map(|entries| {
                        entries
                            .into_iter()
                            .filter(|entry| entry.is_dir && (show || !entry.hidden))
                            .map(|entry| (entry.name.clone(), path.join(&entry.name)))
                            .collect()
                    })
                    .unwrap_or_default();

                items.sort_by_key(|(name, _)| name.to_lowercase());
                self.crumb_menu = Some(CrumbMenu { key: index, items });
            }
            Message::ToggleShowHidden => {
                self.show_hidden = !self.show_hidden;
                self.crumb_menu = None;
                self.refresh();
            }
            Message::ToggleHidden => {
                self.menu_open = None;

                if self
                    .crumb_menu
                    .as_ref()
                    .is_some_and(|menu| menu.key == HIDDEN_KEY)
                {
                    self.crumb_menu = None;
                    return;
                }

                let items = self
                    .current_dir
                    .as_deref()
                    .map(|dir| {
                        let crumbs = path_crumbs(dir);
                        let total = crumbs.len();

                        if total > MAX_CRUMBS {
                            crumbs[1..total - 3].to_vec()
                        } else {
                            Vec::new()
                        }
                    })
                    .unwrap_or_default();

                self.crumb_menu = Some(CrumbMenu {
                    key: HIDDEN_KEY,
                    items,
                });
            }
            Message::Search(text) => {
                self.crumb_menu = None;
                self.search = text;
                self.apply_search();
            }
        }
    }

    /// Clickable path. Every part navigates to that folder, the chevron after
    /// a part opens a list of its subfolders, and long paths collapse to
    /// `first > … > last three` where `…` lists the hidden parents.
    fn breadcrumbs(&self) -> Element<'_, Message> {
        let Some(dir) = &self.current_dir else {
            return text("").into();
        };

        let crumbs = path_crumbs(dir);
        let total = crumbs.len();
        let collapse = total > MAX_CRUMBS;
        let mut bar = Row::new().spacing(2).align_y(Alignment::Center);

        for (i, (label, path)) in crumbs.into_iter().enumerate() {
            let hidden_middle = collapse && i > 0 && i < total - 3;

            if hidden_middle {
                if i == total - 4 {
                    bar = bar.push(
                        button(text("…").size(14))
                            .padding([2, 6])
                            .on_press(Message::ToggleHidden)
                            .style(button::text),
                    );
                }
                continue;
            }

            bar = bar.push(
                button(text(label).size(14).wrapping(Wrapping::None))
                    .padding([2, 6])
                    .on_press(Message::NavigateTo(path.clone()))
                    .style(button::text),
            );

            bar = bar.push(
                button(icons::chevron_right(12.0).render(false, false))
                    .padding([4, 4])
                    .on_press(Message::ToggleChildren(i, path))
                    .style(button::text),
            );
        }

        container(bar).width(Length::Fill).clip(true).into()
    }

    /// Dropdown under the header listing subfolders (or the hidden parents).
    fn crumb_menu_view(&self) -> Option<Element<'_, Message>> {
        let menu = self.crumb_menu.as_ref()?;

        let body: Element<'_, Message> = if menu.items.is_empty() {
            container(text("No subfolders").size(13)).padding(6).into()
        } else {
            let mut list = Column::new().spacing(2);

            for (name, path) in &menu.items {
                list = list.push(
                    button(
                        row![
                            icons::folder(16.0).render(false, false),
                            text(name.as_str()).size(13).wrapping(Wrapping::None),
                        ]
                        .spacing(8)
                        .align_y(Alignment::Center),
                    )
                    .width(Length::Fill)
                    .padding([4, 8])
                    .on_press(Message::NavigateTo(path.clone()))
                    .style(button::text),
                );
            }

            scrollable(list).height(Length::Shrink).into()
        };

        Some(
            container(body)
                .width(Length::Fixed(260.0))
                .max_height(240)
                .padding(4)
                .style(container::rounded_box)
                .into(),
        )
    }

    /// Eye button: highlighted while hidden items are shown.
    fn hidden_toggle(&self) -> Element<'_, Message> {
        let show = self.show_hidden;

        let icon = if show {
            icons::eye(18.0)
        } else {
            icons::eye_off(18.0)
        };

        let toggle = button(icon.render(false, false))
            .padding(6)
            .on_press(Message::ToggleShowHidden)
            .style(move |theme, status| {
                if show {
                    button::primary(theme, status)
                } else {
                    button::text(theme, status)
                }
            });

        tooltip(
            toggle,
            container(text(if show { "Hide hidden items" } else { "Show hidden items" }).size(13))
                .padding(6)
                .style(container::rounded_box),
            tooltip::Position::Bottom,
        )
        .into()
    }

    /// Full-pane layer holding the right-click menu at `menu_pos`.
    /// Clicking anywhere outside the menu closes it.
    fn item_menu_layer(&self, index: usize) -> Element<'_, Message> {
        let on_action: MenuCallback<'static, Message> = Rc::new(Message::MenuAction);
        let pos = self.menu_pos;

        mouse_area(
            container(column![
                Space::with_height(Length::Fixed(pos.y)),
                row![
                    Space::with_width(Length::Fixed(pos.x)),
                    menu_content(index, &on_action),
                ],
            ])
            .width(Length::Fill)
            .height(Length::Fill),
        )
        .on_press(Message::CloseMenu)
        .on_right_press(Message::CloseMenu)
        .into()
    }

    fn main_pane(&self) -> Element<'_, Message> {
        let parent_exists = self
            .current_dir
            .as_ref()
            .and_then(|dir| dir.parent())
            .is_some();

        let header = row![
            button(icons::arrow_up(18.0).render(false, false))
                .padding(6)
                .on_press_maybe(parent_exists.then_some(Message::GoUp))
                .style(button::text),
            self.breadcrumbs(),
            self.hidden_toggle(),
            mode_button(
                icons::layout_list(18.0).render(false, false),
                ViewMode::List,
                self.view_mode == ViewMode::List,
            ),
            mode_button(
                icons::layout_grid(18.0).render(false, false),
                ViewMode::BigTiles,
                self.view_mode == ViewMode::BigTiles,
            ),
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        let mut content = column![header].spacing(12);

        if let Some(menu) = self.crumb_menu_view() {
            content = content.push(menu);
        }

        if let Some(status) = &self.status {
            content = content.push(text(status.as_str()).size(13).style(text::danger));
        }

        if self.current_dir.is_some() {
            if self.entries.is_empty() && !self.search.trim().is_empty() {
                content = content.push(
                    text(format!("No items match “{}”", self.search.trim())).size(14),
                );
            } else {
                let menu = ItemMenu {
                    open: None, // the popup is drawn by `item_menu_layer` instead
                    on_open: Rc::new(Message::OpenMenu),
                    on_action: Rc::new(Message::MenuAction),
                    on_dismiss: Message::CloseMenu,
                };

                content = content.push(
                    FileList::new(&self.entries)
                        .selected(self.list_selected)
                        .on_click(Message::ListClicked)
                        .view(self.view_mode, Some(menu)),
                );
            }
        } else {
            content = content.push(text("Select a folder in the sidebar").size(14));
        }

        let base = mouse_area(
            container(content)
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(16),
        )
        .on_move(Message::CursorMoved);

        match self.menu_open.filter(|&i| i < self.entries.len()) {
            Some(index) => stack![base, self.item_menu_layer(index)].into(),
            None => base.into(),
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        row![self.sidebar.view().map(Message::Sidebar), self.main_pane()].into()
    }
}