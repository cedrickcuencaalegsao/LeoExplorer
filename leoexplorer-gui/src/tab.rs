use crate::sidebar::Sidebar;

pub struct Tab {
    pub sidebar: Sidebar,
}

impl Tab {
    pub fn new(dark: bool) -> Self {
        Self {
            sidebar: Sidebar {
                dark,
                selected: Some(0),
                ..Sidebar::default()
            },
        }
    }

    pub fn title(&self) -> String {
        self.sidebar
            .selected
            .and_then(|i| self.sidebar.sections.get(i))
            .map(|section| section.name.to_string())
            .unwrap_or_else(|| "New Tab".to_string())
    }
}
