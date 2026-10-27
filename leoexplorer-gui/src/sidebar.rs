use iced::widget::{button, column, container, horizontal_space, row, scrollable, text, Column};
use iced::{Alignment, Element, Length};

use crate::drive::Drive;
use crate::icons::{self, Icon};
use crate::spitem::SpItem;
use crate::spmenu::SpMenu;

#[derive(Debug, Clone)]
pub enum Message {
    ToggleSection(usize),
    ToggleTheme,
    OpenEntry(usize, usize),
    Eject(usize),
}

pub struct Entry {
    pub name: String,
    pub is_dir: bool,
    pub hidden: bool,
    pub permission_required: bool,
}

impl Entry {
    pub fn new(name: impl Into<String>, is_dir: bool) -> Self {
        Self {
            name: name.into(),
            is_dir,
            hidden: false,
            permission_required: false,
        }
    }

    pub fn hidden(mut self) -> Self {
        self.hidden = true;
        self
    }

    pub fn locked(mut self) -> Self {
        self.permission_required = true;
        self
    }
}

pub struct Section {
    pub icon: fn(f32) -> Icon,
    pub name: &'static str,
    pub expanded: bool,
    pub hidden: bool,
    pub permission_required: bool,
    pub children: Vec<Entry>,
}

impl Section {
    pub fn new(icon: fn(f32) -> Icon, name: &'static str) -> Self {
        Self {
            icon,
            name,
            expanded: false,
            hidden: false,
            permission_required: false,
            children: Vec::new(),
        }
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

impl Default for Sidebar {
    fn default() -> Self {
        Self {
            dark: false,
            selected: Some(2),
            sections: vec![
                Section::new(icons::house, "Home"),
                Section::new(icons::monitor, "Desktop").children(vec![
                    Entry::new("Proposal", true),
                    Entry::new("Files and...", true),
                    Entry::new("Document.docx", false),
                    Entry::new("Text.txt", false),
                ]),
                Section::new(icons::files, "Documents").children(vec![
                    Entry::new("Proposal", true),
                    Entry::new("Files and...", true),
                    Entry::new("Document.docx", false),
                    Entry::new("Text.txt", false),
                ]),
                Section::new(icons::download, "Downloads"),
                Section::new(icons::headphones, "Music"),
                Section::new(icons::image, "Pictures"),
                Section::new(icons::square_play, "Videos"),
            ],
            drives: vec![
                Drive::new("Local Disk (C:)", 100.0, 153.0),
                Drive::new("New Volume (D:)", 100.0, 153.0),
                Drive::new("USB Drive (E:)", 100.0, 153.0).removable(),
            ],
        }
    }
}

impl Sidebar {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::ToggleSection(i) => {
                self.sections[i].expanded = !self.sections[i].expanded;
                self.selected = Some(i);
            }
            Message::ToggleTheme => self.dark = !self.dark,
            Message::OpenEntry(_, _) => {}
            Message::Eject(_) => {}
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let theme_icon = if self.dark { icons::sun(20.0) } else { icons::moon(20.0) };

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
            let mut menu = SpMenu::new((section.icon)(20.0), section.name)
                .active(self.selected == Some(i))
                .expanded(section.expanded)
                .hidden(section.hidden)
                .permission_required(section.permission_required)
                .on_click(move || Message::ToggleSection(i));

            for (j, entry) in section.children.iter().enumerate() {
                let entry_icon = if entry.is_dir { icons::folder(18.0) } else { icons::file_text(18.0) };

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
            column![header, scrollable(nav).height(Length::Fill), drives].spacing(16),
        )
        .width(280)
        .height(Length::Fill)
        .padding(16)
        .into()
    }
}