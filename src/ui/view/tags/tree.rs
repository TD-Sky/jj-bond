use bytestring::ByteString;
use ratatui::{
    crossterm::event::{KeyCode, KeyModifiers},
    style::{Modifier, Style},
    widgets::Padding,
};
use ratzgo::{
    core::*,
    scroll::ScrollAction,
    text::{Line, Text},
    widget::{BorderType, ListState, MountPoint, block, list},
};
use smol_str::SmolStr;

use crate::{
    ui::{
        Message, TagPush, TagsMsg,
        widgets::{Modal, Tree, TreeState, modal_area},
    },
    utils::tui::TreeText,
};

#[derive(Debug)]
pub struct VState<'a> {
    pub state: &'a mut TreeState<ByteString>,
    pub view: &'a TreeText,
    pub mount_point: &'a MountPoint<Message>,
    pub modal_delete: Option<&'a str>,
    pub modal_push: Option<&'a TagPush>,
    pub modal_remotes: Option<(&'a [SmolStr], &'a mut ListState)>,
}

pub fn view<'a>(
    VState {
        state,
        view,
        modal_delete,
        modal_push,
        mount_point,
        modal_remotes,
    }: VState<'a>,
) -> impl Into<Element<'a, TagsMsg>> {
    if let Some(tag) = modal_delete {
        mount_point.mount(
            Modal::new(" Delete Tag ", format!("delete tag `{tag}` ?"))
                .on_key(
                    |k| k.code == KeyCode::Char('y'),
                    TagsMsg::DeleteConfirm(true).into(),
                )
                .on_key(
                    |k| k.code == KeyCode::Char('n') || k.code == KeyCode::Esc,
                    TagsMsg::DeleteConfirm(false).into(),
                )
                .on_key(|k| k.code == KeyCode::Char('?'), TagsMsg::Help.into()),
            modal_area,
        );
    } else if let Some(push) = modal_push {
        mount_point.mount(
            Modal::new(
                " Push Tag ",
                format!("push tag `{}` to `{}` ?", push.name, push.remote),
            )
            .on_key(
                |k| k.code == KeyCode::Char('y'),
                TagsMsg::PushConfirm(true).into(),
            )
            .on_key(
                |k| k.code == KeyCode::Char('n') || k.code == KeyCode::Esc,
                TagsMsg::PushConfirm(false).into(),
            )
            .on_key(|k| k.code == KeyCode::Char('?'), TagsMsg::Help.into()),
            modal_area,
        );
    } else if let Some((remotes, state)) = modal_remotes {
        let inner = list(state)
            .items(remotes.iter().map(|v| v.as_str()).collect::<Text<'_>>())
            .decorate(|v| v.highlight_style(Style::default().add_modifier(Modifier::REVERSED)))
            .active(true)
            .on_key_with(|k| {
                let msg = match k.code {
                    KeyCode::Char('k') => TagsMsg::ScrollRemotes(ScrollAction::Fixed(-1)),
                    KeyCode::Char('j') => TagsMsg::ScrollRemotes(ScrollAction::Fixed(1)),
                    KeyCode::Char('d') if k.modifiers == KeyModifiers::CONTROL => {
                        TagsMsg::ScrollRemotes(ScrollAction::Viewport(50))
                    }
                    KeyCode::Char('u') if k.modifiers == KeyModifiers::CONTROL => {
                        TagsMsg::ScrollRemotes(ScrollAction::Viewport(-50))
                    }
                    KeyCode::Char('f') if k.modifiers == KeyModifiers::CONTROL => {
                        TagsMsg::ScrollRemotes(ScrollAction::Viewport(100))
                    }
                    KeyCode::Char('b') if k.modifiers == KeyModifiers::CONTROL => {
                        TagsMsg::ScrollRemotes(ScrollAction::Viewport(-100))
                    }
                    KeyCode::Enter => TagsMsg::TrackConfirm(true),
                    KeyCode::Esc => TagsMsg::TrackConfirm(false),
                    KeyCode::Char('?') => TagsMsg::Help,
                    _ => return None,
                };
                Some(msg.into())
            });

        mount_point.mount(
            block(inner)
                .bordered()
                .border_type(BorderType::Rounded)
                .title(Line::from(" Track ").centered()),
            modal_area,
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
                KeyCode::Char('t') => TagsMsg::Track,
                KeyCode::Char('u') => TagsMsg::Untrack,
                KeyCode::Char('d') => TagsMsg::Delete,
                KeyCode::Char('p') => TagsMsg::Push,
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
