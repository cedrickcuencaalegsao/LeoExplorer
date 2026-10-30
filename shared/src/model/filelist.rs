use super::Entry;
use crate::types::RowCallback;
use std::rc::Rc;

pub struct FileList<'a, Message> {
    pub entries: &'a [Entry],
    pub selected: Option<usize>,
    pub on_click: Option<RowCallback<'a, Message>>,
}

impl<'a, Message: 'a> FileList<'a, Message> {
    pub fn new(entries: &'a [Entry]) -> Self {
        Self {
            entries,
            selected: None,
            on_click: None,
        }
    }

    pub fn selected(mut self, selected: Option<usize>) -> Self {
        self.selected = selected;
        self
    }

    pub fn on_click(mut self, callback: impl Fn(usize) -> Message + 'a) -> Self {
        self.on_click = Some(Rc::new(callback));
        self
    }
}
