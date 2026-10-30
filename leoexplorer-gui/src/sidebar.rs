use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use iced::widget::{Column, button, column, container, horizontal_space, row, scrollable, text};
use iced::{Alignment, Element, Length, Theme};
use shared::{AllDirectories, Entry};

use crate::drive::Drive;
use crate::icons::{self, Icon};
use crate::spitem::SpItem;
use crate::spmenu::SpMenu;

const SECTIONS: &[(&str, &str, fn(f32) -> Icon)] = &[
    ("Home", "Home", icons::house),
    ("Desktop", "Desktop", icons::monitor),
    ("Documents", "Documents", icons::files),
    ("Downloads", "Downloads", icons::download),
    ("Music", "Music", icons::headphones),
    ("Pictures", "Pictures", icons::image),
    ("Movies", "Videos", icons::square_play),
];

const HIDDEN_VOLUMES: &[&str] = &["Recovery", "Preboot", "VM", "Update"];

#[derive(Debug, Clone)]
pub enum Message {
    ToggleSection(usize),
    ToggleTheme,
    OpenEntry(usize, usize),
    Eject(usize),
}



pub struct Section {
    pub icon: fn(f32) -> Icon,
    pub name: String,
    pub path: Option<PathBuf>,
    pub expanded: bool,
    pub hidden: bool,
    pub permission_required: bool,
    pub children: Vec<Entry>,
}

impl Section {
    pub fn new(icon: fn(f32) -> Icon, name: impl Into<String>) -> Self {
        Self {
            icon,
            name: name.into(),
            path: None,
            expanded: false,
            hidden: false,
            permission_required: false,
            children: Vec::new(),
        }
    }

    pub fn path(mut self, path: PathBuf) -> Self {
        self.path = Some(path);
        self
    }

    pub fn expanded(mut self) -> Self {
        self.expanded = true;
        self
    }

    pub fn locked(mut self) -> Self {
        self.permission_required = true;
        self
    }

    pub fn children(mut self, children: Vec<Entry>) -> Self {
        self.children = children;
        self
    }
}

pub struct Sidebar {
    pub sections: Vec<Section>,
    pub drives: Vec<Drive>,
    pub dark: bool,
    pub selected: Option<usize>,
}

pub(crate) fn read_children(path: &Path) -> io::Result<Vec<Entry>> {
    let mut entries = Vec::new();

    for item in fs::read_dir(path)?.flatten() {
        let name = item.file_name().to_string_lossy().into_owned();
        let is_dir = item.file_type().map(|t| t.is_dir()).unwrap_or(false);
        let modified = item.metadata().ok().and_then(|meta| meta.modified().ok());

        let mut entry = Entry::new(name.clone(), is_dir)
            .path(item.path())
            .modified(modified);

        if name.starts_with('.') {
            entry = entry.hidden();
        }
        entries.push(entry);
    }

    entries.sort_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });

    Ok(entries)
}

pub(crate) fn scrollbar_style(theme: &Theme, status: scrollable::Status) -> scrollable::Style {
    let mut style = scrollable::default(theme, status);

    style.vertical_rail.background = None;
    style.vertical_rail.border.radius = 2.0.into();
    style.vertical_rail.scroller.border.radius = 2.0.into();

    style
}

pub(crate) fn thin_scrollbar() -> scrollable::Direction {
    scrollable::Direction::Vertical(
        scrollable::Scrollbar::new()
            .width(4.0)
            .scroller_width(4.0)
            .margin(2.0),
    )
}

impl Sidebar {
    pub fn from_system(data: &AllDirectories) -> Self {
        let mut sections = Vec::new();

        for (key, label, icon) in SECTIONS {
            if let Some(path) = data.special_dirs.get(*key) {
                sections.push(Section::new(*icon, *label).path(path.clone()));
            }
        }

        for (name, path) in &data.icloud_dirs {
            sections.push(Section::new(icons::cloud, name.clone()).path(path.clone()));
        }

        let mut drives = Vec::new();

        for info in &data.drives {
            if HIDDEN_VOLUMES.contains(&info.name.as_str()) {
                continue;
            }

            let mut drive = Drive::new(
                info.name.clone(),
                info.used_space_gb as f32,
                info.total_space_gb as f32,
            );

            if format!("{:?}", info.drive_type) == "Removable" {
                drive = drive.removable();
            }

            drives.push(drive);
        }

        Self {
            sections,
            drives,
            dark: false,
            selected: Some(0),
        }
    }

    pub fn section_path(&self, i: usize) -> Option<PathBuf> {
        self.sections.get(i)?.path.clone()
    }

    pub fn entry_path(&self, i: usize, j: usize) -> Option<(PathBuf, bool)> {
        let section = self.sections.get(i)?;
        let entry = section.children.get(j)?;
        Some((section.path.as_ref()?.join(&entry.name), entry.is_dir))
    }

    pub fn reload(&mut self) {
        for section in &mut self.sections {
            if !section.expanded {
                continue;
            }

            if let Some(path) = &section.path {
                match read_children(path) {
                    Ok(children) => section.children = children,
                    Err(err) => log::warn!("cannot read {}: {err}", path.display()),
                }
            }
        }
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::ToggleSection(i) => {
                let Some(section) = self.sections.get_mut(i) else {
                    return;
                };

                section.expanded = !section.expanded;

                if section.expanded {
                    if let Some(path) = &section.path {
                        match read_children(path) {
                            Ok(children) => section.children = children,
                            Err(err) => {
                                log::warn!("cannot read {}: {err}", path.display());
                                if err.kind() == io::ErrorKind::PermissionDenied {
                                    section.permission_required = true;
                                }
                            }
                        }
                    }
                }

                self.selected = Some(i);
            }
            Message::ToggleTheme => self.dark = !self.dark,
            Message::OpenEntry(_, _) => {}
            Message::Eject(_) => {}
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let theme_icon = if self.dark {
            icons::sun(20.0)
        } else {
            icons::moon(20.0)
        };

        let header = row![
            icons::rocket(28.0),
            text("Leo Explorer").size(22),
            horizontal_space(),
            button(theme_icon)
                .on_press(Message::ToggleTheme)
                .style(button::text),
        ]
        .spacing(10)
        .align_y(Alignment::Center);

        let mut nav = Column::new().spacing(2);

        for (i, section) in self.sections.iter().enumerate() {
            let mut menu = SpMenu::new((section.icon)(20.0), section.name.clone())
                .active(self.selected == Some(i))
                .expanded(section.expanded)
                .expandable(section.path.is_some())
                .hidden(section.hidden)
                .permission_required(section.permission_required)
                .on_click(move || Message::ToggleSection(i));

            for (j, entry) in section.children.iter().enumerate() {
                let entry_icon = if entry.is_dir {
                    icons::folder(18.0)
                } else {
                    icons::file_text(18.0)
                };

                menu = menu.child(
                    SpItem::new(entry_icon, entry.name.clone())
                        .hidden(entry.hidden)
                        .permission_required(entry.permission_required)
                        .on_click(move || Message::OpenEntry(i, j)),
                );
            }

            nav = nav.push(menu);
        }

        let mut drives = Column::new().spacing(14);

        for (i, drive) in self.drives.iter().enumerate() {
            drives = drives.push(drive.view(move || Message::Eject(i)));
        }

        container(
            column![
                header,
                scrollable(nav)
                    .height(Length::Fill)
                    .direction(thin_scrollbar())
                    .style(scrollbar_style),
                drives
            ]
            .spacing(16),
        )
        .width(280)
        .height(Length::Fill)
        .padding(16)
        .into()
    }
}