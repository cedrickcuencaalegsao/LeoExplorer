use shared::AllDirectories;

use crate::sidebar::Sidebar;

pub struct Tab {
    pub sidebar: Sidebar,
}

impl Tab {
    pub fn new(dark: bool, dirs: &AllDirectories) -> Self {
        let mut sidebar = Sidebar::from_system(dirs);
        sidebar.dark = dark;
        Self { sidebar }
    }

    pub fn title(&self) -> String {
        self.sidebar
            .selected
            .and_then(|i| self.sidebar.sections.get(i))
            .map(|section| section.name.clone())
            .unwrap_or_else(|| "New Tab".to_string())
    }
}