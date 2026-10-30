use std::rc::Rc;

use iced::widget::{Column, button, container, mouse_area, text};
use iced::{Background, Border, Color, Element, Length, Theme};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemAction {
    Open,
    Rename,
    Copy,
    Delete,
    Properties,
}

impl ItemAction {
    pub const ALL: [ItemAction; 5] = [
        ItemAction::Open,
        ItemAction::Rename,
        ItemAction::Copy,
        ItemAction::Delete,
        ItemAction::Properties,
    ];

    fn label(self) -> &'static str {
        match self {
            ItemAction::Open => "Open",
            ItemAction::Rename => "Rename",
            ItemAction::Copy => "Copy",
            ItemAction::Delete => "Delete",
            ItemAction::Properties => "Properties",
        }
    }
}

pub type MenuCallback<'a, Message> = Rc<dyn Fn(usize, ItemAction) -> Message + 'a>;
pub type OpenCallback<'a, Message> = Rc<dyn Fn(usize) -> Message + 'a>;

/// Callbacks (and state) for the item popup menu.
///
/// The menu itself is drawn by the parent view (see `Tab::item_menu_layer`)
/// so it can be placed at the mouse cursor. This struct only wires up the
/// right-click on each item.
pub struct ItemMenu<'a, Message> {
    /// Index of the entry whose menu is open. Not used by `with_menu` any
    /// more; kept so existing call sites keep compiling.
    pub open: Option<usize>,
    /// Fired when an entry is right-clicked.
    pub on_open: OpenCallback<'a, Message>,
    /// Fired when a menu entry is chosen.
    pub on_action: MenuCallback<'a, Message>,
    /// Fired when the user clicks outside the menu. Not used by `with_menu`
    /// any more; the parent's overlay layer handles dismissal.
    pub on_dismiss: Message,
}

fn menu_button_style(danger: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |theme, status| {
        let palette = theme.extended_palette();
        let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);

        let text_color = if danger {
            palette.danger.base.color
        } else {
            palette.background.base.text
        };

        button::Style {
            background: hovered.then(|| Background::Color(palette.background.weak.color)),
            text_color,
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 6.0.into(),
            },
            ..button::Style::default()
        }
    }
}

/// The menu panel itself (fixed 180px wide).
pub fn menu_content<'a, Message: Clone + 'a>(
    index: usize,
    on_action: &MenuCallback<'a, Message>,
) -> Element<'a, Message> {
    let mut items = Column::new()
        .spacing(2)
        .padding(4)
        .width(Length::Fixed(180.0));

    for action in ItemAction::ALL {
        items = items.push(
            button(text(action.label()).size(13))
                .width(Length::Fill)
                .padding([6, 10])
                .style(menu_button_style(action == ItemAction::Delete))
                .on_press(on_action(index, action)),
        );
    }

    container(items).style(container::rounded_box).into()
}

/// Wraps an item so a right-click on it reports `on_open(index)`.
///
/// `button` ignores the right mouse button, so the click falls through to
/// the `mouse_area`. `_width` is unused but kept so `filelist.rs` doesn't
/// need to change.
pub fn with_menu<'a, Message: Clone + 'a>(
    item: Element<'a, Message>,
    index: usize,
    menu: Option<&ItemMenu<'a, Message>>,
    _width: Length,
) -> Element<'a, Message> {
    let Some(menu) = menu else {
        return item;
    };

    mouse_area(item)
        .on_right_press((menu.on_open)(index))
        .into()
}