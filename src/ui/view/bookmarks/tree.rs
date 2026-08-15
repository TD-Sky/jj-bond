use bytestring::ByteString;
use ratatui::{
    crossterm::event::{KeyCode, KeyModifiers},
    layout::Constraint,
    style::{Modifier, Style},
    text::Text,
    widgets::Padding,
};
use ratzgo::{
    core::*,
    scroll::ScrollAction,
    text::Line,
    widget::{BorderType, ListState, MountPoint, block, list},
};
use smol_str::SmolStr;

use crate::{
    ui::{
        BookmarksMsg, Message,
        widgets::{Modal, Tree, TreeState},
    },
    utils::tui::TreeText,
};

pub struct VState<'a> {
    pub view: &'a TreeText,
    pub state: &'a mut TreeState<ByteString>,
    pub mount_point: &'a MountPoint<Message>,
    pub modal_delete: Option<&'a ByteString>,
    pub modal_remotes: Option<(&'a [SmolStr], &'a mut ListState)>,
}

pub fn view<'a>(
    VState {
        view,
        state,
        mount_point,
        modal_delete,
        modal_remotes,
    }: VState<'a>,
) -> impl Into<Element<'a, BookmarksMsg>> {
    if let Some(bookmark) = modal_delete {
        mount_point.mount(
            Modal::new(
                " Delete Bookmark ",
                format!("delete bookmark `{bookmark}` ?"),
            )
            .on_key(
                |k| k.code == KeyCode::Char('y'),
                BookmarksMsg::DeleteConfirm(true).into(),
            )
            .on_key(
                |k| k.code == KeyCode::Char('n') || k.code == KeyCode::Esc,
                BookmarksMsg::DeleteConfirm(false).into(),
            )
            .on_key(|k| k.code == KeyCode::Char('?'), BookmarksMsg::Help.into()),
            |area| area.centered(Constraint::Ratio(1, 2), Constraint::Ratio(1, 3)),
        );
    } else if let Some((remotes, state)) = modal_remotes {
        let inner = list(state)
            .items(remotes.iter().map(|v| v.as_str()).collect::<Text<'_>>())
            .decorate(|v| v.highlight_style(Style::default().add_modifier(Modifier::REVERSED)))
            .active(true)
            .on_key_with(|k| {
                let msg = match k.code {
                    KeyCode::Char('k') => BookmarksMsg::ScrollRemotes(ScrollAction::Fixed(-1)),
                    KeyCode::Char('j') => BookmarksMsg::ScrollRemotes(ScrollAction::Fixed(1)),
                    KeyCode::Char('d') if k.modifiers == KeyModifiers::CONTROL => {
                        BookmarksMsg::ScrollRemotes(ScrollAction::Viewport(50))
                    }
                    KeyCode::Char('u') if k.modifiers == KeyModifiers::CONTROL => {
                        BookmarksMsg::ScrollRemotes(ScrollAction::Viewport(-50))
                    }
                    KeyCode::Char('f') if k.modifiers == KeyModifiers::CONTROL => {
                        BookmarksMsg::ScrollRemotes(ScrollAction::Viewport(100))
                    }
                    KeyCode::Char('b') if k.modifiers == KeyModifiers::CONTROL => {
                        BookmarksMsg::ScrollRemotes(ScrollAction::Viewport(-100))
                    }
                    KeyCode::Enter => BookmarksMsg::TrackConfirm(true),
                    KeyCode::Esc => BookmarksMsg::TrackConfirm(false),
                    KeyCode::Char('?') => BookmarksMsg::Help,
                    _ => return None,
                };
                Some(msg.into())
            });

        mount_point.mount(
            block(inner)
                .bordered()
                .border_type(BorderType::Rounded)
                .title(Line::from(" Track ").centered()),
            |area| area.centered(Constraint::Ratio(1, 2), Constraint::Ratio(1, 3)),
        );
    }

    let inner = Tree::new(view.get(), state)
        .active(true)
        .decorate(|v| v.highlight_style(Style::default().add_modifier(Modifier::REVERSED)))
        .on_key_with(|k| {
            let msg = match k.code {
                KeyCode::Char('k') => BookmarksMsg::ScrollTree(ScrollAction::Fixed(-1)),
                KeyCode::Char('j') => BookmarksMsg::ScrollTree(ScrollAction::Fixed(1)),
                KeyCode::Char('d') if k.modifiers == KeyModifiers::CONTROL => {
                    BookmarksMsg::ScrollTree(ScrollAction::Viewport(50))
                }
                KeyCode::Char('u') if k.modifiers == KeyModifiers::CONTROL => {
                    BookmarksMsg::ScrollTree(ScrollAction::Viewport(-50))
                }
                KeyCode::Char('f') if k.modifiers == KeyModifiers::CONTROL => {
                    BookmarksMsg::ScrollTree(ScrollAction::Viewport(100))
                }
                KeyCode::Char('b') if k.modifiers == KeyModifiers::CONTROL => {
                    BookmarksMsg::ScrollTree(ScrollAction::Viewport(-100))
                }
                KeyCode::Char('l') => BookmarksMsg::BookmarkOpen,
                KeyCode::Char('h') => BookmarksMsg::BookmarkClose,
                KeyCode::Char('K') if k.modifiers == KeyModifiers::SHIFT => {
                    BookmarksMsg::ScrollHistory(ScrollAction::Fixed(-1))
                }
                KeyCode::Char('J') if k.modifiers == KeyModifiers::SHIFT => {
                    BookmarksMsg::ScrollHistory(ScrollAction::Fixed(1))
                }
                KeyCode::Char('t') => BookmarksMsg::Track,
                KeyCode::Char('u') => BookmarksMsg::Untrack,
                KeyCode::Char('d') => BookmarksMsg::Delete,
                KeyCode::Enter => BookmarksMsg::ViewHistory,
                KeyCode::Char('?') => BookmarksMsg::Help,
                _ => return None,
            };
            Some(msg)
        });

    block(inner)
        .bordered()
        .border_type(BorderType::Rounded)
        .decorate(|v| v.padding(Padding::horizontal(1)))
}
