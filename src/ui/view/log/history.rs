use ratatui::{
    crossterm::event::{KeyCode, KeyModifiers},
    macros::constraint,
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

use crate::{
    ui::{
        LogMsg, Message,
        view::log::{LogFocus, LogLayout, bookmark_list, tag_list},
        widgets::{LogHistory, LogHistoryState, Modal, TextAreaState, modal_area, rebase},
    },
    utils::{
        jj::{Abandon, Duplicate, LogMode, Rebase, Split, SplitMode, Squash},
        tui::{BoxText, LogText},
    },
};

#[derive(Debug)]
pub struct VState<'a> {
    pub state: &'a mut LogHistoryState,
    pub view: &'a LogText,
    pub log_focus: &'a LogFocus,
    pub log_layout: &'a LogLayout,
    pub log_mode: &'a LogMode,
    pub mount_point: &'a MountPoint<Message>,
    pub modal_abandon: Option<&'a Abandon>,
    pub modal_squash: Option<&'a Squash>,
    pub modal_rebase: Option<&'a Rebase>,
    pub modal_rebase_list: Option<&'a Text<'a>>,
    pub modal_rebase_list_state: (&'a mut ListState, &'a mut ListState),
    pub modal_rebase_from: Option<&'a str>,
    pub modal_split: Option<&'a Split>,
    pub modal_duplicate: Option<&'a Duplicate>,
    pub modal_bookmark_list: Option<&'a Text<'a>>,
    pub modal_bookmark_list_state: &'a mut ListState,
    pub modal_bookmark_list_input: Option<&'a mut TextAreaState>,
    pub modal_tag_list: Option<&'a Text<'a>>,
    pub modal_tag_list_state: &'a mut ListState,
    pub modal_tag_list_input: Option<&'a mut TextAreaState>,
    pub modal_undo: bool,
    pub modal_redo: bool,
    pub modal_unsync: Option<(&'a BoxText, &'a mut ListState)>,
}

pub fn view<'a>(
    VState {
        state,
        view,
        log_focus,
        log_layout,
        log_mode,
        mount_point,
        modal_abandon,
        modal_squash,
        modal_rebase,
        modal_rebase_list,
        modal_rebase_list_state,
        modal_rebase_from,
        modal_split,
        modal_duplicate,
        modal_bookmark_list,
        modal_bookmark_list_state,
        modal_bookmark_list_input,
        modal_tag_list,
        modal_tag_list_state,
        modal_tag_list_input,
        modal_undo,
        modal_redo,
        modal_unsync,
    }: VState<'a>,
) -> impl Into<Element<'a, LogMsg>> {
    let hover = state.hovered();

    if let Some(v) = modal_abandon {
        mount_point.mount(
            Modal::new(" Abandon ", v.msg())
                .on_key(
                    |k| k.code == KeyCode::Char('y'),
                    LogMsg::AbandonConfirm(true).into(),
                )
                .on_key(
                    |k| k.code == KeyCode::Char('n') || k.code == KeyCode::Esc,
                    LogMsg::AbandonConfirm(false).into(),
                )
                .on_key(|k| k.code == KeyCode::Char('?'), LogMsg::Help.into()),
            modal_area,
        );
    } else if let Some(squash) = modal_squash {
        mount_point.mount(
            Modal::new(" Squash ", squash.msg())
                .on_key(
                    |k| k.code == KeyCode::Char('y'),
                    LogMsg::SquashConfirm(true).into(),
                )
                .on_key(
                    |k| k.code == KeyCode::Char('n') || k.code == KeyCode::Esc,
                    LogMsg::SquashConfirm(false).into(),
                )
                .on_key(|k| k.code == KeyCode::Char('?'), LogMsg::Help.into()),
            modal_area,
        );
    } else if let Some(text) = modal_rebase_list {
        let (from_state, to_state) = modal_rebase_list_state;
        mount_point.mount(
            rebase::view(rebase::VState {
                view: text,
                from_state,
                to_state,
                from: modal_rebase_from,
            })
            .into()
            .map(Into::into),
            |area| area.centered(constraint!(==50%), constraint!(==50%)),
        );
    } else if let Some(rebase) = modal_rebase {
        mount_point.mount(
            Modal::new(" Rebase ", rebase.msg())
                .on_key(
                    |k| k.code == KeyCode::Char('y'),
                    LogMsg::RebaseConfirm(true).into(),
                )
                .on_key(
                    |k| k.code == KeyCode::Char('n') || k.code == KeyCode::Esc,
                    LogMsg::RebaseConfirm(false).into(),
                )
                .on_key(|k| k.code == KeyCode::Char('?'), LogMsg::Help.into()),
            modal_area,
        );
    } else if let Some(dup) = modal_duplicate {
        mount_point.mount(
            Modal::new(" Duplicate ", dup.msg())
                .on_key(
                    |k| k.code == KeyCode::Char('y'),
                    LogMsg::DuplicateConfirm(true).into(),
                )
                .on_key(
                    |k| k.code == KeyCode::Char('n') || k.code == KeyCode::Esc,
                    LogMsg::DuplicateConfirm(false).into(),
                )
                .on_key(|k| k.code == KeyCode::Char('?'), LogMsg::Help.into()),
            modal_area,
        );
    } else if let Some(Split { id, mode }) = modal_split {
        let content = match mode {
            SplitMode::ParentChild => format!("split {id}"),
            SplitMode::Parallel => format!("split {id} parallelly"),
        };

        mount_point.mount(
            Modal::new(" Split ", content)
                .on_key(
                    |k| k.code == KeyCode::Char('y'),
                    LogMsg::SplitConfirm(true).into(),
                )
                .on_key(
                    |k| k.code == KeyCode::Char('n') || k.code == KeyCode::Esc,
                    LogMsg::SplitConfirm(false).into(),
                )
                .on_key(|k| k.code == KeyCode::Char('?'), LogMsg::Help.into()),
            modal_area,
        );
    } else if let Some(text) = modal_bookmark_list {
        let list = bookmark_list::view(bookmark_list::VState {
            view: text,
            state: modal_bookmark_list_state,
            input: modal_bookmark_list_input,
        });
        mount_point.mount(list, |area| {
            area.centered(constraint!(==40%), constraint!(==40%))
        });
    } else if let Some(text) = modal_tag_list {
        let list = tag_list::view(tag_list::VState {
            view: text,
            state: modal_tag_list_state,
            input: modal_tag_list_input,
        });
        mount_point.mount(list, |area| {
            area.centered(constraint!(==40%), constraint!(==40%))
        });
    } else if modal_undo {
        mount_point.mount(
            Modal::new(" Undo ", "undo an operation?")
                .on_key(
                    |k| k.code == KeyCode::Char('y'),
                    LogMsg::UndoConfirm(true).into(),
                )
                .on_key(
                    |k| k.code == KeyCode::Char('n') || k.code == KeyCode::Esc,
                    LogMsg::UndoConfirm(false).into(),
                )
                .on_key(|k| k.code == KeyCode::Char('?'), LogMsg::Help.into()),
            modal_area,
        );
    } else if modal_redo {
        mount_point.mount(
            Modal::new(" Redo ", "redo an operation?")
                .on_key(
                    |k| k.code == KeyCode::Char('y'),
                    LogMsg::RedoConfirm(true).into(),
                )
                .on_key(
                    |k| k.code == KeyCode::Char('n') || k.code == KeyCode::Esc,
                    LogMsg::RedoConfirm(false).into(),
                )
                .on_key(|k| k.code == KeyCode::Char('?'), LogMsg::Help.into()),
            modal_area,
        );
    } else if let Some((bookmarks, state)) = modal_unsync {
        let inner = list(state)
            .items(bookmarks.get())
            .decorate(|v| v.highlight_style(Style::default().add_modifier(Modifier::REVERSED)))
            .active(true)
            .on_key_with(|k| {
                let msg = match k.code {
                    KeyCode::Char('k') => LogMsg::ScrollUnsync(ScrollAction::Fixed(-1)),
                    KeyCode::Char('j') => LogMsg::ScrollUnsync(ScrollAction::Fixed(1)),
                    KeyCode::Char('d') if k.modifiers == KeyModifiers::CONTROL => {
                        LogMsg::ScrollUnsync(ScrollAction::Viewport(50))
                    }
                    KeyCode::Char('u') if k.modifiers == KeyModifiers::CONTROL => {
                        LogMsg::ScrollUnsync(ScrollAction::Viewport(-50))
                    }
                    KeyCode::Char('f') if k.modifiers == KeyModifiers::CONTROL => {
                        LogMsg::ScrollUnsync(ScrollAction::Viewport(100))
                    }
                    KeyCode::Char('b') if k.modifiers == KeyModifiers::CONTROL => {
                        LogMsg::ScrollUnsync(ScrollAction::Viewport(-100))
                    }
                    KeyCode::Enter => LogMsg::PushConfirm(true),
                    KeyCode::Esc => LogMsg::PushConfirm(false),
                    KeyCode::Char('?') => LogMsg::Help,
                    _ => return None,
                };
                Some(msg.into())
            });

        mount_point.mount(
            block(inner)
                .bordered()
                .border_type(BorderType::Rounded)
                .title(Line::from(" Push bookmark ").centered()),
            modal_area,
        );
    }
    let yanking = state.yanking().is_some();

    let inner = LogHistory::new(view, state)
        .active(log_focus.is_history())
        .on_key_with(move |k| {
            let msg = match k.code {
                KeyCode::Char('k') => LogMsg::ScrollHistory(ScrollAction::Fixed(-1)),
                KeyCode::Char('j') => LogMsg::ScrollHistory(ScrollAction::Fixed(1)),
                KeyCode::Char('d') if k.modifiers == KeyModifiers::CONTROL => {
                    LogMsg::ScrollHistory(ScrollAction::Viewport(50))
                }
                KeyCode::Char('d') if yanking && let Some(change) = view.beacons().get(hover) => {
                    LogMsg::Duplicate {
                        id: change.id.clone(),
                    }
                }
                KeyCode::Char('d') if let Some(change) = view.beacons().get(hover) => {
                    LogMsg::Desc {
                        id: change.id.clone(),
                    }
                }
                KeyCode::Char('u') if k.modifiers == KeyModifiers::CONTROL => {
                    LogMsg::ScrollHistory(ScrollAction::Viewport(-50))
                }
                KeyCode::Char('u') => LogMsg::Undo,
                KeyCode::Char('r') if k.modifiers == KeyModifiers::CONTROL => LogMsg::Redo,
                KeyCode::Char('r') if yanking && let Some(change) = view.beacons().get(hover) => {
                    LogMsg::Rebase {
                        id: change.id.clone(),
                    }
                }
                KeyCode::Char('r') => LogMsg::RebaseListOpen,
                KeyCode::Char('f') if k.modifiers == KeyModifiers::CONTROL => {
                    LogMsg::ScrollHistory(ScrollAction::Viewport(100))
                }
                KeyCode::Char('b') if k.modifiers == KeyModifiers::CONTROL => {
                    LogMsg::ScrollHistory(ScrollAction::Viewport(-100))
                }
                KeyCode::Enter if *log_layout == LogLayout::HISTORY => {
                    LogMsg::Layout(LogLayout::HISTORY_FILES)
                }
                KeyCode::Enter if *log_layout == LogLayout::HISTORY_FILES => {
                    LogMsg::Layout(LogLayout::FILES_DIFF)
                }
                KeyCode::Esc if yanking => LogMsg::Unyank,
                KeyCode::Esc if *log_layout == LogLayout::HISTORY_FILES => {
                    LogMsg::Layout(LogLayout::HISTORY)
                }
                KeyCode::Esc
                    if *log_mode != LogMode::Default && *log_layout == LogLayout::HISTORY =>
                {
                    LogMsg::ResetMode
                }
                KeyCode::Char('K')
                    if k.modifiers == KeyModifiers::SHIFT
                        && *log_layout == LogLayout::HISTORY_FILES =>
                {
                    LogMsg::ScrollShow(ScrollAction::Fixed(-1))
                }
                KeyCode::Char('J')
                    if k.modifiers == KeyModifiers::SHIFT
                        && *log_layout == LogLayout::HISTORY_FILES =>
                {
                    LogMsg::ScrollShow(ScrollAction::Fixed(1))
                }
                KeyCode::Char('n') if let Some(change) = view.beacons().get(hover) => LogMsg::New {
                    parent: change.id.clone(),
                },
                KeyCode::Char('e') if let Some(change) = view.beacons().get(hover) => {
                    LogMsg::Edit {
                        id: change.id.clone(),
                    }
                }
                KeyCode::Char('a') if let Some(change) = view.beacons().get(hover) => {
                    LogMsg::Abandon {
                        id: change.id.clone(),
                    }
                }
                KeyCode::Char('s') if let Some(change) = view.beacons().get(hover) => {
                    LogMsg::Squash {
                        id: change.id.clone(),
                    }
                }
                KeyCode::Char('x') if let Some(change) = view.beacons().get(hover) => {
                    LogMsg::Split(Split {
                        id: change.id.clone(),
                        mode: SplitMode::ParentChild,
                    })
                }
                KeyCode::Char('X') if let Some(change) = view.beacons().get(hover) => {
                    LogMsg::Split(Split {
                        id: change.id.clone(),
                        mode: SplitMode::Parallel,
                    })
                }
                KeyCode::Char('f') => LogMsg::Fetch,
                KeyCode::Char('b') => LogMsg::BookmarkListOpen,
                KeyCode::Char('t') => LogMsg::TagListOpen,
                KeyCode::Char('p') => LogMsg::Push,
                KeyCode::Char(' ') if let Some(change) = view.beacons().get(hover) => {
                    LogMsg::Yank {
                        id: change.id.clone(),
                    }
                }
                KeyCode::Char('?') => LogMsg::Help,
                _ => return None,
            };
            Some(msg)
        });

    block(inner)
        .bordered()
        .border_type(BorderType::Rounded)
        .decorate(|v| v.padding(Padding::horizontal(1)))
}
