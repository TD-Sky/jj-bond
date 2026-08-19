use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui_textarea::CursorMove;
use ratzgo::{
    core::Element,
    event::{DefaultContext, YieldFg},
    widget::row,
};
use smol_str::SmolStr;

use crate::{
    ui::{
        HelpMsg, LogMsg, LogRelocate, MainState, Message, State,
        view::log::{LogLayout, diff, files, history, show},
        widgets::{TextAreaState, Yanking},
    },
    utils::{
        jj::{Abandon, Duplicate, LogMode, Rebase, Squash},
        tui::{BoxText, LogText},
    },
};

pub fn view<'a>(state: &'a mut MainState) -> Element<'a, LogMsg> {
    let css = state.log_layout.constraints();

    if state.log_layout == LogLayout::HISTORY {
        history::view(history::VState {
            state: &mut state.log_history_state,
            view: &state.log_history,
            log_focus: &state.log_focus,
            log_layout: &state.log_layout,
            log_mode: &state.log_mode,
            mount_point: &state.mount_point,
            modal_abandon: state.log_abandon.as_ref(),
            modal_squash: state.log_squash.as_ref(),
            modal_rebase: state.log_rebase.as_ref(),
            modal_rebase_list: state.log_modal_rebase_list.as_deref(),
            modal_rebase_list_state: (
                &mut state.log_modal_rebase_list_state.0,
                &mut state.log_modal_rebase_list_state.1,
            ),
            modal_rebase_from: state.log_modal_rebase_from.as_deref(),
            modal_duplicate: state.log_duplicate.as_ref(),
            modal_split: state.log_split.as_ref(),
            modal_bookmark_list: state.log_modal_bookmark_list.as_deref(),
            modal_bookmark_list_state: &mut state.log_modal_bookmark_list_state.0,
            modal_bookmark_list_input: state.log_modal_bookmark_list_state.1.as_mut(),
            modal_tag_list: state.log_modal_tag_list.as_deref(),
            modal_tag_list_state: &mut state.log_modal_tag_list_state.0,
            modal_tag_list_input: state.log_modal_tag_list_state.1.as_mut(),
            modal_undo: state.log_modal_undo_state,
            modal_redo: state.log_modal_redo_state,
            modal_unsync: state
                .log_modal_unsync
                .as_ref()
                .map(|v| (v, &mut state.log_modal_unsync_state)),
        })
        .into()
    } else if state.log_layout == LogLayout::HISTORY_FILES {
        row! [
            css;
            [
                history::view(history::VState {
                    state: &mut state.log_history_state,
                    view: &state.log_history,
                    log_focus: &state.log_focus,
                    log_layout: &state.log_layout,
                    log_mode: &state.log_mode,
                    mount_point: &state.mount_point,
                    modal_abandon: state.log_abandon.as_ref(),
                    modal_squash: state.log_squash.as_ref(),
                    modal_rebase: state.log_rebase.as_ref(),
                    modal_rebase_list: state.log_modal_rebase_list.as_deref(),
                    modal_rebase_list_state: (
                        &mut state.log_modal_rebase_list_state.0,
                        &mut state.log_modal_rebase_list_state.1,
                    ),
                    modal_rebase_from: state.log_modal_rebase_from.as_deref(),
                    modal_duplicate: state.log_duplicate.as_ref(),
                    modal_split: state.log_split.as_ref(),
                    modal_bookmark_list: state.log_modal_bookmark_list.as_deref(),
                    modal_bookmark_list_state: &mut state.log_modal_bookmark_list_state.0,
                    modal_bookmark_list_input: state.log_modal_bookmark_list_state.1.as_mut(),
                    modal_tag_list: state.log_modal_tag_list.as_deref(),
                    modal_tag_list_state: &mut state.log_modal_tag_list_state.0,
                    modal_tag_list_input: state.log_modal_tag_list_state.1.as_mut(),
                    modal_undo: state.log_modal_undo_state,
                    modal_redo: state.log_modal_redo_state,
                    modal_unsync: state
                        .log_modal_unsync
                        .as_ref()
                        .map(|v| (v, &mut state.log_modal_unsync_state)),
                }),
                show::view(show::VState {
                    state: &mut state.log_show_state,
                    view: state.log_show_view.get(),
                }),
            ]
        ]
        .into()
    } else if state.log_layout == LogLayout::FILES_DIFF {
        row! [
            css;
            [
                files::view(files::VState {
                    state: &mut state.log_files_state,
                    log_focus: &state.log_focus,
                    view: state.log_files_view.get(),
                    id: state
                        .log_history
                        .beacons()
                        .get(state.log_history_state.hovered())
                        .map(|v| v.id.as_str()),
                }),
                diff::view(diff::VState {
                    state: &mut state.log_diff_state,
                    log_focus: &state.log_focus,
                    view: state.log_diff_view.get(),
                    id: None,
                    file: None,
                }),
            ]
        ]
        .into()
    } else if state.log_layout == LogLayout::DIFF {
        diff::view(diff::VState {
            state: &mut state.log_diff_state,
            log_focus: &state.log_focus,
            view: state.log_diff_view.get(),
            id: state
                .log_history
                .beacons()
                .get(state.log_history_state.hovered())
                .map(|v| v.id.as_str()),
            file: state.log_reloc.file(),
        })
        .into()
    } else {
        unreachable!()
    }
}

pub async fn update(state: &mut MainState, msg: LogMsg, ctx: &mut DefaultContext<Message, State>) {
    match msg {
        LogMsg::UpdateHistory(log) => {
            state.log_history = log;

            let id = match &mut state.log_reloc {
                LogRelocate::Concrete { id, file } => {
                    match state.log_history.find_by_id(id) {
                        Some((i, _)) => {
                            state.log_history_state.hover(i);
                        }
                        None => {
                            let Some((i, change)) = state.log_history.find_working() else {
                                return;
                            };
                            state.log_history_state.hover(i);

                            *id = change.id.clone();
                            file.take();
                            state.log_diff_state.reset();
                        }
                    };
                    id.clone()
                }
                LogRelocate::Working => {
                    let Some((i, change)) = state.log_history.find_working() else {
                        return;
                    };
                    state.log_history_state.hover(i);

                    state.log_reloc = LogRelocate::Concrete {
                        id: change.id.clone(),
                        file: None,
                    };
                    state.log_diff_state.reset();

                    change.id.clone()
                }
                LogRelocate::Index { index, file } => {
                    let (i, change) = match state.log_history.beacons().get(*index) {
                        Some(change) => (*index, change),
                        None => {
                            let Some(pair) = state.log_history.find_working() else {
                                return;
                            };

                            state.log_diff_state.reset();
                            file.take();

                            pair
                        }
                    };

                    state.log_history_state.hover(i);
                    state.log_reloc = LogRelocate::Concrete {
                        id: change.id.clone(),
                        file: file.take(),
                    };

                    change.id.clone()
                }
            };

            if state.log_layout == LogLayout::HISTORY_FILES {
                debounce_show(state, id);
            } else if state.log_layout == LogLayout::FILES_DIFF {
                let jj = state.jj_handle.clone();
                ctx.queue()
                    .spawn_try(async move { jj.diff_sum(&id).await.map(LogMsg::UpdateFiles) });
            }
        }
        LogMsg::ScrollHistory(action) => {
            let Some(change) = state
                .log_history_state
                .scroll_vertical(&state.log_history, action)
            else {
                return;
            };

            state.log_reloc = LogRelocate::Concrete {
                id: change.id.clone(),
                file: None,
            };
            state.log_diff_state.reset();

            debounce_show(state, change.id.clone());
        }
        LogMsg::UpdateShow { text, version } => {
            if state.log_show_debounce().version() != version
                && let LogRelocate::Concrete { id, .. } = &state.log_reloc
            {
                debounce_show(state, id.clone());
            } else {
                state.log_show_view = text.into();
            }
        }
        LogMsg::Layout(layout) => {
            state.log_layout = layout;
            state.log_focus = state.log_layout.into();
            if let Some(change) = state
                .log_history
                .beacons()
                .get(state.log_history_state.hovered())
            {
                if state.log_layout == LogLayout::HISTORY_FILES {
                    debounce_show(state, change.id.clone());
                    state.log_files_state.select_first();
                } else if state.log_layout == LogLayout::FILES_DIFF {
                    let change_id = change.id.clone();
                    let jj = state.jj_handle.clone();
                    ctx.queue().spawn_try(async move {
                        jj.diff_sum(&change_id).await.map(LogMsg::UpdateFiles)
                    });
                }
            }
        }
        LogMsg::UpdateFiles(s) => {
            let LogRelocate::Concrete {
                id,
                file: file_reloc,
            } = &mut state.log_reloc
            else {
                return;
            };

            state.log_files_view = s.into();

            if let Some(file) = file_reloc
                && let Some(i) = state
                    .log_files_view
                    .iter()
                    .enumerate()
                    .find_map(|(i, line)| {
                        line.iter()
                            .any(|span| span.content == file.as_str())
                            .then_some(i)
                    })
            {
                state.log_files_state.select(Some(i));
            } else {
                state.log_files_state.reset();
                *file_reloc = state
                    .log_files_view
                    .lines
                    .first()
                    .and_then(|v| v.spans.first().map(|v| v.content.as_ref().into()));
                state.log_diff_state.reset();
            }

            let diff = file_reloc.as_ref().map(|file| (id.clone(), file.clone()));
            if let Some((id, file)) = diff {
                debounce_diff(state, id, file);
            } else {
                state.log_diff_debounce_mut().cancel();
                state.log_diff_view = Default::default();
            }
        }
        LogMsg::UpdateDiff { text, version } => {
            if state.log_diff_debounce().version() != version
                && let LogRelocate::Concrete {
                    id,
                    file: Some(status_file),
                } = &state.log_reloc
            {
                debounce_diff(state, id.clone(), status_file.clone());
            } else {
                state.log_diff_view = text.into();
            }
        }
        LogMsg::ScrollDiff(action) => {
            state
                .log_diff_state
                .scroll_vertical(action, state.log_diff_view.height());
        }
        LogMsg::ScrollHDiff(action) => {
            state
                .log_diff_state
                .scroll_horizontal(action, state.log_diff_view.width());
        }
        LogMsg::ScrollShow(action) => {
            state
                .log_show_state
                .scroll_vertical(action, state.log_show_view.height());
        }
        LogMsg::ScrollFiles(action) => {
            let LogRelocate::Concrete {
                file: file_reloc, ..
            } = &mut state.log_reloc
            else {
                return;
            };

            state
                .log_files_state
                .scroll_lines(action, state.log_files_view.height());

            if let Some(i) = state.log_files_state.selected()
                && let Some(file) = state.log_files_view.lines.get(i).map(|v| {
                    v.spans
                        .first()
                        .map(|v| v.content.as_ref())
                        .unwrap_or_default()
                })
            {
                *file_reloc = Some(file.into());
                state.log_diff_state.reset();

                if let Some(change) = state
                    .log_history
                    .beacons()
                    .get(state.log_history_state.hovered())
                {
                    debounce_diff(state, change.id.clone(), file.into());
                }
            }
        }
        LogMsg::New { parent } => match state.jj_handle.new(&parent).await {
            Ok(_) => {
                state.log_reloc = LogRelocate::Working;
                state.log_mode = LogMode::Default;
            }
            Err(e) => {
                ratzgo::log::error("`new`", e.into_text());
            }
        },
        LogMsg::Edit { id } => {
            if let Err(e) = state.jj_handle.edit(&id).await {
                ratzgo::log::error("`edit`", e.into_text());
            }
        }
        LogMsg::Desc { id } => {
            ctx.set_fg(YieldFg::new_ignore(async move |state: &mut State, _| {
                if let Err(e) = state.main.jj_handle.desc(&id).await {
                    ratzgo::log::error("`desc`", e.into_text());
                }
            }));
        }
        LogMsg::Abandon { id } => {
            let v = match state.log_history_state.yanking() {
                Some(Yanking::One { id }) => Abandon::One { id: id.clone() },
                Some(Yanking::Range {
                    base: (start, end), ..
                }) => Abandon::Range {
                    start: start.clone(),
                    end: end.clone(),
                },
                None => Abandon::One { id },
            };
            state.log_abandon = Some(v);
        }
        LogMsg::AbandonConfirm(yes) => {
            if let Some(v) = state.log_abandon.take()
                && yes
            {
                state.log_history_state.unyank();

                match state.jj_handle.abandon(&v).await {
                    Ok(_) => {
                        if let LogRelocate::Concrete { file, .. } = &mut state.log_reloc {
                            state.log_reloc = LogRelocate::Index {
                                index: state.log_history_state.hovered(),
                                file: file.take(),
                            }
                        }
                    }
                    Err(e) => {
                        ratzgo::log::error("`abandon`", e.into_text());
                    }
                }
            }
        }
        LogMsg::Squash { id } => {
            state.log_squash = match state.log_history_state.yanking() {
                Some(Yanking::One { id: from }) => match from.split_once('/') {
                    Some((change_id, change_offset))
                        if let Some((short_id, divergent)) = id.split_once('/')
                            && (!change_id.starts_with(short_id) || change_offset != divergent) =>
                    {
                        Some(Squash::OneTo {
                            from: from.clone(),
                            to: id,
                        })
                    }
                    None if !from.starts_with(id.as_str()) => Some(Squash::OneTo {
                        from: from.clone(),
                        to: id,
                    }),
                    _ => None,
                },
                Some(Yanking::Range {
                    base: (start, end), ..
                }) => Some(Squash::RangeTo {
                    start: start.clone(),
                    end: end.clone(),
                    to: id,
                }),
                None if let Some((_, change)) = state.log_history.find_working() => {
                    Some(Squash::ToParent {
                        id: change.id.clone(),
                    })
                }
                None => None,
            };
        }
        LogMsg::SquashConfirm(yes) => {
            if let Some(v) = state.log_squash.take()
                && yes
            {
                state.log_history_state.unyank();

                let v = YieldFg::new_ignore(async move |state: &mut State, _| {
                    match state.main.jj_handle.squash(&v).await {
                        Ok(_) => {
                            state.main.log_reloc = match v {
                                Squash::ToParent { .. } => LogRelocate::Working,
                                Squash::ToStart { start: to, .. }
                                | Squash::OneTo { to, .. }
                                | Squash::RangeTo { to, .. } => {
                                    LogRelocate::Concrete { id: to, file: None }
                                }
                            };
                        }
                        Err(e) => {
                            ratzgo::log::error("`squash`", e.into_text());
                        }
                    }
                });
                ctx.set_fg(v);
            }
        }
        LogMsg::Split(v) => state.log_split = Some(v),
        LogMsg::SplitConfirm(yes) => {
            if let Some(v) = state.log_split.take()
                && yes
            {
                ctx.set_fg(YieldFg::new_ignore(
                    async move |state: &mut State, _| match state.main.jj_handle.split(&v).await {
                        Ok(_) => {
                            if let LogRelocate::Concrete { file, .. } = &mut state.main.log_reloc {
                                state.main.log_reloc = LogRelocate::Index {
                                    index: state.main.log_history_state.hovered(),
                                    file: file.take(),
                                }
                            }
                        }
                        Err(e) => {
                            ratzgo::log::error("`split`", e.into_text());
                        }
                    },
                ));
            }
        }
        LogMsg::Fetch => {
            if let Ok(remotes) = state.jj_handle.remotes().await
                && !remotes.is_empty()
                && !*state.log_fetching.borrow()
            {
                let fetcing = state.log_fetching.clone();
                *fetcing.borrow() = true;
                let jj_handle = state.jj_handle.clone();

                ctx.queue().spawn(async move {
                    let res = jj_handle.fetch().await;
                    *fetcing.borrow() = false;
                    if let Err(e) = res {
                        ratzgo::log::error("`git fetch`", e.into_text());
                    }
                    Message::Refresh // force update anyway
                });
            }
        }
        LogMsg::BookmarkListOpen => {
            let s = match state.jj_handle.bookmarks().await {
                Ok(s) => s,
                Err(e) => {
                    ratzgo::log::error("`bookmark list`", e.into_text());
                    return;
                }
            };

            let mut view: BoxText = s.into();
            view.lines.dedup_by(|lhs, rhs| {
                lhs.spans.first().map(|v| v.content.as_ref())
                    == rhs.spans.first().map(|v| v.content.as_ref())
            });
            state.log_modal_bookmark_list = Some(view);

            state.log_modal_bookmark_list_state.0.reset();
        }
        LogMsg::BookmarkListClose => {
            state.log_modal_bookmark_list = None;
            state.log_modal_bookmark_list_state.0.select(None);
        }
        LogMsg::BookmarkListSelect => {
            if let Some(list) = &state.log_modal_bookmark_list {
                match state.log_modal_bookmark_list_state.0.selected() {
                    Some(0) => {
                        state.log_modal_bookmark_list_state.1 = Some(TextAreaState::default());
                    }
                    Some(i) => {
                        let LogRelocate::Concrete { id, .. } = &mut state.log_reloc else {
                            return;
                        };

                        let i_list = i - 1;
                        let Some(name) =
                            list.lines.get(i_list).map(|v| v.spans[0].content.as_ref())
                        else {
                            state.log_modal_bookmark_list = None;
                            state.log_modal_bookmark_list_state.0.select(None);

                            return;
                        };

                        let res = state.jj_handle.bookmark_set(id, name).await;

                        state.log_modal_bookmark_list = None;
                        state.log_modal_bookmark_list_state.0.select(None);

                        if let Err(e) = res {
                            ratzgo::log::error("`bookmark set`", e.into_text());
                        }
                    }
                    _ => (),
                }
            }
        }
        LogMsg::BookmarkListScroll(action) => {
            if let Some(list) = &state.log_modal_bookmark_list {
                state
                    .log_modal_bookmark_list_state
                    .0
                    .scroll_lines(action, list.height() + 1);
            }
        }
        LogMsg::CreatingBookmark { key } => {
            let LogRelocate::Concrete { id, .. } = &mut state.log_reloc else {
                return;
            };

            let Some(input) = &mut state.log_modal_bookmark_list_state.1 else {
                return;
            };

            if key.modifiers.is_empty() {
                match key.code {
                    KeyCode::Char(c) => input.insert_char(c),
                    KeyCode::Enter
                        if let Some(name) = input.lines().first().filter(|v| !v.is_empty()) =>
                    {
                        let res = state.jj_handle.bookmark_create(id, name.trim()).await;
                        state.log_modal_bookmark_list_state.1.take();
                        state.log_modal_bookmark_list = None;
                        state.log_modal_bookmark_list_state.0.select(None);
                        if let Err(e) = res {
                            ratzgo::log::error("`bookmark create`", e.into_text());
                        }
                    }
                    KeyCode::Esc => {
                        state.log_modal_bookmark_list_state.1.take();
                    }
                    KeyCode::Backspace => {
                        input.move_cursor(CursorMove::Forward);
                        input.delete_char();
                    }
                    _ => (),
                }
            } else if key.modifiers == KeyModifiers::CONTROL {
                match key.code {
                    KeyCode::Char('u') => {
                        input.clear();
                    }
                    KeyCode::Char('b') => input.move_cursor(CursorMove::Back),
                    KeyCode::Char('f') => input.move_cursor(CursorMove::Forward),
                    KeyCode::Char('a') => input.move_cursor(CursorMove::Head),
                    KeyCode::Char('e') => input.move_cursor(CursorMove::End),
                    _ => (),
                }
            }
        }
        LogMsg::Undo => {
            state.log_modal_undo_state = true;
        }
        LogMsg::UndoConfirm(yes) => {
            state.log_modal_undo_state = false;

            if yes {
                match state.jj_handle.undo().await {
                    Ok(_) => {
                        state.log_reloc = LogRelocate::Index {
                            index: state.log_history_state.hovered(),
                            file: None,
                        };
                        state.log_mode = LogMode::Default;
                    }
                    Err(e) => {
                        ratzgo::log::error("`undo`", e.into_text());
                    }
                }
            }
        }
        LogMsg::Redo => {
            state.log_modal_redo_state = true;
        }
        LogMsg::RedoConfirm(yes) => {
            state.log_modal_redo_state = false;

            if yes {
                match state.jj_handle.redo().await {
                    Ok(_) => {
                        state.log_reloc = LogRelocate::Index {
                            index: state.log_history_state.hovered(),
                            file: None,
                        };
                        state.log_mode = LogMode::Default;
                    }
                    Err(e) => {
                        ratzgo::log::error("`redo`", e.into_text());
                    }
                }
            }
        }
        LogMsg::Yank { id } => {
            state
                .log_history_state
                .yank(&state.log_history, &state.jj_handle, &id)
                .await;
        }
        LogMsg::Unyank => {
            state.log_history_state.unyank();
        }
        LogMsg::RebaseListOpen => {
            let s = match state.jj_handle.bookmarks().await {
                Ok(s) => s,
                Err(e) => {
                    ratzgo::log::error("`bookmark list`", e.into_text());
                    return;
                }
            };

            let mut view: BoxText = s.into();
            view.lines
                .dedup_by(|lhs, rhs| lhs.spans[0].content == rhs.spans[0].content);

            state.log_modal_rebase_list = Some(view);
            state.log_modal_rebase_from = None;
            state.log_modal_rebase_list_state.1.select(None);
            if state
                .log_modal_rebase_list
                .as_ref()
                .is_some_and(|view| !view.lines.is_empty())
            {
                state.log_modal_rebase_list_state.0.reset();
            } else {
                state.log_modal_rebase_list_state.0.select(None);
            }
        }
        LogMsg::RebaseListClose => {
            close_rebase_list(state);
        }
        LogMsg::RebaseListBack => {
            state.log_modal_rebase_from = None;
            state.log_modal_rebase_list_state.1.select(None);
        }
        LogMsg::RebaseListSelect => {
            if let Some(from) = state.log_modal_rebase_from.clone() {
                let to = state.log_modal_rebase_list.as_ref().and_then(|view| {
                    state
                        .log_modal_rebase_list_state
                        .1
                        .selected()
                        .and_then(|i| {
                            view.lines
                                .iter()
                                .filter(|line| line.spans[0].content != from.as_str())
                                .nth(i)
                                .map(|v| v.spans[0].content.as_ref())
                        })
                });
                let Some(to) = to else {
                    return;
                };

                state.log_rebase = Some(Rebase::One {
                    from,
                    to: to.into(),
                });
                state.log_rebase_clear_yank = false;
                close_rebase_list(state);
            } else {
                let from = state.log_modal_rebase_list.as_ref().and_then(|view| {
                    state
                        .log_modal_rebase_list_state
                        .0
                        .selected()
                        .and_then(|i| view.lines.get(i))
                        .map(|v| v.spans[0].content.as_ref())
                });
                let Some(from) = from else {
                    return;
                };

                state.log_modal_rebase_from = Some(from.into());
                if state
                    .log_modal_rebase_list
                    .as_ref()
                    .is_some_and(|view| view.lines.len() > 1)
                {
                    state.log_modal_rebase_list_state.1.reset();
                } else {
                    state.log_modal_rebase_list_state.1.select(None);
                }
            }
        }
        LogMsg::RebaseListScroll(action) => {
            let Some(view) = state.log_modal_rebase_list.as_ref() else {
                return;
            };

            if let Some(from) = state.log_modal_rebase_from.as_deref() {
                let height = view
                    .lines
                    .iter()
                    .filter(|line| line.spans[0].content != from)
                    .count();
                state
                    .log_modal_rebase_list_state
                    .1
                    .scroll_lines(action, height);
            } else {
                state
                    .log_modal_rebase_list_state
                    .0
                    .scroll_lines(action, view.lines.len());
            }
        }
        LogMsg::Rebase { id } => {
            state.log_rebase = match state.log_history_state.yanking() {
                Some(Yanking::One { id: from }) => match from.split_once('/') {
                    Some((change_id, change_offset))
                        if let Some((short_id, divergent)) = id.split_once('/')
                            && (!change_id.starts_with(short_id) || change_offset != divergent) =>
                    {
                        Some(Rebase::One {
                            from: from.clone(),
                            to: id,
                        })
                    }
                    None if !from.starts_with(id.as_str()) => Some(Rebase::One {
                        from: from.clone(),
                        to: id,
                    }),
                    _ => None,
                },
                Some(Yanking::Range {
                    base: (start, end),
                    ids,
                }) => {
                    match ids
                        .lines()
                        .all(|change_id| match change_id.split_once('/') {
                            Some((change_id, change_offset))
                                if let Some((short_id, divergent)) = id.split_once('/')
                                    && (!change_id.starts_with(short_id)
                                        || change_offset != divergent) =>
                            {
                                true
                            }
                            None if !change_id.starts_with(id.as_str()) => true,
                            _ => false,
                        }) {
                        true => Some(Rebase::Range {
                            start: start.clone(),
                            end: end.clone(),
                            to: id.clone(),
                        }),
                        false => None,
                    }
                }
                None => None,
            };
            state.log_rebase_clear_yank = state.log_rebase.is_some();
        }
        LogMsg::RebaseConfirm(yes) => {
            let clear_yank = std::mem::take(&mut state.log_rebase_clear_yank);
            if let Some(v) = state.log_rebase.take()
                && yes
            {
                if clear_yank {
                    state.log_history_state.unyank();
                }

                match state.jj_handle.rebase(&v).await {
                    Ok(_) => {
                        state.log_reloc = LogRelocate::Concrete {
                            id: v.reloc().into(),
                            file: None,
                        };
                    }
                    Err(e) => {
                        ratzgo::log::error("`rebase`", e.into_text());
                    }
                }
            }
        }
        LogMsg::Duplicate { id } => {
            state.log_duplicate = match state.log_history_state.yanking() {
                Some(Yanking::One { id: from }) => match from.split_once('/') {
                    Some((change_id, change_offset))
                        if let Some((short_id, divergent)) = id.split_once('/')
                            && (!change_id.starts_with(short_id) || change_offset != divergent) =>
                    {
                        Some(Duplicate::One {
                            from: from.clone(),
                            to: id,
                        })
                    }
                    None if !from.starts_with(id.as_str()) => Some(Duplicate::One {
                        from: from.clone(),
                        to: id,
                    }),
                    _ => None,
                },
                Some(Yanking::Range {
                    base: (start, end),
                    ids,
                }) => {
                    match ids
                        .lines()
                        .all(|change_id| match change_id.split_once('/') {
                            Some((change_id, change_offset))
                                if let Some((short_id, divergent)) = id.split_once('/')
                                    && (!change_id.starts_with(short_id)
                                        || change_offset != divergent) =>
                            {
                                true
                            }
                            None if !change_id.starts_with(id.as_str()) => true,
                            _ => false,
                        }) {
                        true => Some(Duplicate::Range {
                            start: start.clone(),
                            end: end.clone(),
                            to: id.clone(),
                        }),
                        false => None,
                    }
                }
                None => None,
            };
        }
        LogMsg::DuplicateConfirm(yes) => {
            if let Some(v) = state.log_duplicate.take()
                && yes
            {
                state.log_history_state.unyank();

                match state.jj_handle.duplicate(&v).await {
                    Ok(_) => {
                        state.log_mode = LogMode::Default;
                    }
                    Err(e) => {
                        ratzgo::log::error("`duplicate`", e.into_text());
                    }
                }
            }
        }
        LogMsg::ResetMode => {
            state.log_mode = LogMode::Default;
            state.log_history_state.reset();
            ctx.queue().push(Message::Refresh);
        }
        LogMsg::TagListOpen => {
            let Ok(s) = state.jj_handle.tags().await else {
                return;
            };

            let mut view: BoxText = s.into();
            view.lines.dedup_by(|lhs, rhs| {
                lhs.spans.first().map(|v| v.content.as_ref())
                    == rhs.spans.first().map(|v| v.content.as_ref())
            });
            state.log_modal_tag_list = Some(view);

            state.log_modal_tag_list_state.0.reset();
        }
        LogMsg::TagListClose => {
            state.log_modal_tag_list = None;
            state.log_modal_tag_list_state.0.select(None);
        }
        LogMsg::TagListSelect => {
            if let Some(list) = &state.log_modal_tag_list {
                match state.log_modal_tag_list_state.0.selected() {
                    Some(0) => {
                        state.log_modal_tag_list_state.1 = Some(TextAreaState::default());
                    }
                    Some(i) => {
                        let LogRelocate::Concrete { id, .. } = &mut state.log_reloc else {
                            return;
                        };

                        let i_list = i - 1;
                        let Some(name) =
                            list.lines.get(i_list).map(|v| v.spans[0].content.as_ref())
                        else {
                            state.log_modal_tag_list = None;
                            state.log_modal_tag_list_state.0.select(None);

                            return;
                        };

                        let res = state.jj_handle.tag_set(id, name).await;

                        state.log_modal_tag_list = None;
                        state.log_modal_tag_list_state.0.select(None);

                        if let Err(e) = res {
                            ratzgo::log::error("`tag set`", e.into_text());
                        }
                    }
                    _ => (),
                }
            }
        }
        LogMsg::TagListScroll(action) => {
            if let Some(list) = &state.log_modal_tag_list {
                state
                    .log_modal_tag_list_state
                    .0
                    .scroll_lines(action, list.height() + 1);
            }
        }
        LogMsg::CreatingTag { key } => {
            let LogRelocate::Concrete { id, .. } = &mut state.log_reloc else {
                return;
            };

            let Some(input) = &mut state.log_modal_tag_list_state.1 else {
                return;
            };

            if key.modifiers.is_empty() {
                match key.code {
                    KeyCode::Char(c) => input.insert_char(c),
                    KeyCode::Enter
                        if let Some(name) = input.lines().first().filter(|v| !v.is_empty()) =>
                    {
                        let res = state.jj_handle.tag_set(id, name.trim()).await;
                        state.log_modal_tag_list_state.1.take();
                        state.log_modal_tag_list = None;
                        state.log_modal_tag_list_state.0.select(None);
                        if let Err(e) = res {
                            ratzgo::log::error("`tag set`", e.into_text());
                        }
                    }
                    KeyCode::Esc => {
                        state.log_modal_tag_list_state.1.take();
                    }
                    KeyCode::Backspace => {
                        input.move_cursor(CursorMove::Forward);
                        input.delete_char();
                    }
                    _ => (),
                }
            } else if key.modifiers == KeyModifiers::CONTROL {
                match key.code {
                    KeyCode::Char('u') => {
                        input.clear();
                    }
                    KeyCode::Char('b') => input.move_cursor(CursorMove::Back),
                    KeyCode::Char('f') => input.move_cursor(CursorMove::Forward),
                    KeyCode::Char('a') => input.move_cursor(CursorMove::Head),
                    KeyCode::Char('e') => input.move_cursor(CursorMove::End),
                    _ => (),
                }
            }
        }
        LogMsg::ScrollUnsync(action) => {
            if let Some(v) = &state.log_modal_unsync {
                state
                    .log_modal_unsync_state
                    .scroll_lines(action, v.height());
            }
        }
        LogMsg::Push => {
            if !*state.log_pushing.borrow() {
                match state.jj_handle.bookmarks_unsync().await {
                    Ok(v) if !v.is_empty() => {
                        state.log_modal_unsync = Some(v.into());
                        state.log_modal_unsync_state.reset();
                    }
                    Err(e) => {
                        ratzgo::log::error("`bookmark list`", e.into_text());
                    }
                    _ => (),
                }
            }
        }
        LogMsg::PushConfirm(yes) => {
            if let Some(v) = state.log_modal_unsync.take()
                && yes
                && let Some(i) = state.log_modal_unsync_state.selected()
                && let Some((bookmark, remote)) = v.lines.get(i).and_then(|line| {
                    let bm = line.spans.first()?.content.as_ref();
                    let rm = line.spans[1].content.trim_start_matches(" -> ");
                    Some((SmolStr::new(bm), SmolStr::new(rm)))
                })
            {
                let pushing = state.log_pushing.clone();
                let jj_handle = state.jj_handle.clone();

                compio::runtime::spawn(async move {
                    *pushing.borrow() = true;
                    let res = jj_handle.push_bookmark(&bookmark, &remote).await;
                    *pushing.borrow() = false;
                    if let Err(e) = res {
                        ratzgo::log::error("`push bookmark`", e.into_text());
                    }
                })
                .detach();

                // force update anyway
                ctx.queue().push(Message::Refresh);
            }
        }
        LogMsg::Help => {
            let page = if state.log_abandon.is_some()
                || state.log_squash.is_some()
                || state.log_rebase.is_some()
                || state.log_duplicate.is_some()
                || state.log_split.is_some()
                || state.log_modal_undo_state
                || state.log_modal_redo_state
            {
                "confirm-modal"
            } else if state.log_modal_bookmark_list.is_some() {
                "log-bookmark-list"
            } else if state.log_modal_tag_list.is_some() {
                "log-tag-list"
            } else if state.log_modal_unsync.is_some() {
                "log-unsync"
            } else if state.log_layout == LogLayout::HISTORY {
                "log-history"
            } else if state.log_layout == LogLayout::HISTORY_FILES {
                "log-history-show"
            } else if state.log_layout == LogLayout::FILES_DIFF {
                "log-files-diff"
            } else if state.log_layout == LogLayout::DIFF {
                "log-diff"
            } else {
                unreachable!()
            };

            ctx.queue().push(HelpMsg::Page(page));
        }
        LogMsg::Paste => (),
    }
}

fn close_rebase_list(state: &mut MainState) {
    state.log_modal_rebase_list = None;
    state.log_modal_rebase_from = None;
    state.log_modal_rebase_list_state.0.select(None);
    state.log_modal_rebase_list_state.1.select(None);
}

pub fn refresh(state: &mut MainState, ctx: &mut DefaultContext<Message, State>) {
    let jj_handle = state.jj_handle.clone();
    let mode = state.log_mode.clone();
    ctx.queue().spawn_try(async move {
        jj_handle
            .log(&mode)
            .await
            .map(|v| LogMsg::UpdateHistory(LogText::new(v)))
    });
}

fn debounce_show(state: &mut MainState, id: SmolStr) {
    let jj = state.jj_handle.clone();
    state
        .log_show_debounce_mut()
        .spawn_try(|version| async move {
            jj.show(&id)
                .await
                .map(|text| LogMsg::UpdateShow { text, version })
        });
}

fn debounce_diff(state: &mut MainState, id: SmolStr, status_file: SmolStr) {
    let jj = state.jj_handle.clone();
    state
        .log_diff_debounce_mut()
        .spawn_try(|version| async move {
            let (status, file) = status_file
                .split_once(' ')
                .unwrap_or(("", status_file.as_str()));
            jj.diff(&id, status, file)
                .await
                .map(|text| LogMsg::UpdateDiff { text, version })
        });
}
