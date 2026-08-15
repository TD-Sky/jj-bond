use bytestring::ByteString;
use ratatui::{
    crossterm::event::{KeyCode, KeyModifiers},
    layout::Constraint,
    style::{Modifier, Style},
    widgets::Padding,
};
use ratzgo::{
    core::*,
    scroll::ScrollAction,
    widget::{BorderType, MountPoint, block},
};

use crate::{
    ui::{
        Message, TagsMsg,
        widgets::{Modal, Tree, TreeState},
    },
    utils::tui::TreeText,
};

#[derive(Debug)]
pub struct VState<'a> {
    pub state: &'a mut TreeState<ByteString>,
    pub view: &'a TreeText,
    pub mount_point: &'a MountPoint<Message>,
    pub modal_delete: Option<&'a str>,
}

pub fn view<'a>(
    VState {
        state,
        view,
        modal_delete,
        mount_point,
    }: VState<'a>,
) -> impl Into<Element<'a, TagsMsg>> {
    if let Some(tag) = modal_delete {
        mount_point.mount(
            Modal::new("Delete Tag", format!("delete tag `{tag}` ?"))
                .on_key(
                    |k| k.code == KeyCode::Char('y'),
                    TagsMsg::DeleteConfirm(true).into(),
                )
                .on_key(
                    |k| k.code == KeyCode::Char('n') || k.code == KeyCode::Esc,
                    TagsMsg::DeleteConfirm(false).into(),
                )
                .on_key(|k| k.code == KeyCode::Char('?'), TagsMsg::Help.into()),
            |area| area.centered(Constraint::Ratio(1, 2), Constraint::Ratio(1, 3)),
        );
    }

    let inner = Tree::new(view.get(), state)
        .active(true)
        .decorate(|v| v.highlight_style(Style::new().add_modifier(Modifier::REVERSED)))
        .on_key_with(|k| {
            let msg = match k.code {
                KeyCode::Char('k') => TagsMsg::ScrollTree(ScrollAction::Fixed(-1)),
                KeyCode::Char('j') => TagsMsg::ScrollTree(ScrollAction::Fixed(1)),
                KeyCode::Char('d') if k.modifiers == KeyModifiers::CONTROL => {
                    TagsMsg::ScrollTree(ScrollAction::Viewport(50))
                }
                KeyCode::Char('u') if k.modifiers == KeyModifiers::CONTROL => {
                    TagsMsg::ScrollTree(ScrollAction::Viewport(-50))
                }
                KeyCode::Char('f') if k.modifiers == KeyModifiers::CONTROL => {
                    TagsMsg::ScrollTree(ScrollAction::Viewport(100))
                }
                KeyCode::Char('b') if k.modifiers == KeyModifiers::CONTROL => {
                    TagsMsg::ScrollTree(ScrollAction::Viewport(-100))
                }
                KeyCode::Char('l') => TagsMsg::TagOpen,
                KeyCode::Char('h') => TagsMsg::TagClose,
                KeyCode::Char('K') if k.modifiers == KeyModifiers::SHIFT => {
                    TagsMsg::ScrollHistory(ScrollAction::Fixed(-1))
                }
                KeyCode::Char('J') if k.modifiers == KeyModifiers::SHIFT => {
                    TagsMsg::ScrollHistory(ScrollAction::Fixed(1))
                }
                KeyCode::Char('d') => TagsMsg::Delete,
                KeyCode::Enter => TagsMsg::ViewHistory,
                KeyCode::Char('?') => TagsMsg::Help,
                _ => return None,
            };
            Some(msg)
        });

    block(inner)
        .bordered()
        .border_type(BorderType::Rounded)
        .decorate(|v| v.padding(Padding::horizontal(1)))
}
