use ratatui::{
    crossterm::event::{KeyCode, KeyModifiers},
    text::Text,
};
use ratatui_textarea::CursorMove;
use ratzgo::{
    component::row,
    core::{Component, ComponentExt},
    event::{DefaultContext, YieldFg},
};
use smol_str::SmolStr;

use crate::{
    ui::{
        FilesView, HelpMsg, LogMsg, LogRelocate, MainState, Message, State,
        view::log::{LogLayout, diff, files, history, show},
        widgets::{TextAreaState, Yanking},
    },
    utils::{
        jj::{Abandon, Duplicate, LogMode, Rebase, Squash},
        tui::{BoxText, LogText, PathTree},
    },
};

pub fn view<'a>(state: &'a mut MainState) -> Box<dyn Component<LogMsg> + 'a> {
    let css = state.log_layout.constraints();

    if state.log_layout == LogLayout::HISTORY {
        history::view(history::VState {
            state: &mut state.log_history_state,
            area: &state.log_history_area,
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
        .boxed()
    } else if state.log_layout == LogLayout::HISTORY_FILES {
        row! [
            css;
            [
                history::view(history::VState {
                    state: &mut state.log_history_state,
                    area: &state.log_history_area,
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
                    area: &state.log_show_area,
                    view: state.log_show_view.get(),
                }),
            ]
        ]
        .boxed()
    } else if state.log_layout == LogLayout::FILES_DIFF {
        // the tree can select a directory, which has no diff of its own: blank the preview rather
        // than showing the diff of a file that is not the selected node
        let directory = state.log_files_view == FilesView::Tree
            && state
                .log_files_tree_state
                .selected()
                .last()
                .is_some_and(|path| state.log_files_tree_view.status_file(path).is_none());

        row! [
            css;
            [
                files::view(files::VState {
                    state: &mut state.log_files_state,
                    area: &state.log_files_area,
                    log_focus: &state.log_focus,
                    view: state.log_files_list_view.get(),
                    tab: state.log_files_view,
                    tree_view: &state.log_files_tree_view,
                    tree_state: &mut state.log_files_tree_state,
                    id: state
                        .log_history
                        .beacons()
                        .get(state.log_history_state.hovered())
                        .map(|v| v.id.as_str()),
                }),
                diff::view(diff::VState {
                    state: &mut state.log_diff_state,
                    area: None,
                    log_focus: &state.log_focus,
                    view: if directory {
                        Text::default()
                    } else {
                        state.log_diff_view.get()
                    },
                    id: None,
                    file: None,
                }),
            ]
        ]
        .boxed()
    } else if state.log_layout == LogLayout::DIFF {
        diff::view(diff::VState {
            state: &mut state.log_diff_state,
            area: Some(&state.log_diff_area),
            log_focus: &state.log_focus,
            view: state.log_diff_view.get(),
            id: state
                .log_history
                .beacons()
                .get(state.log_history_state.hovered())
                .map(|v| v.id.as_str()),
            file: state.log_reloc.file(),
        })
        .boxed()
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
                spawn_files_list(state, ctx, id.clone());
                debounce_files_tree(state, ctx, id);
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
                    spawn_files_list(state, ctx, change_id.clone());
                    debounce_files_tree(state, ctx, change_id);
                }
            }
        }
        LogMsg::FilesViewSelect(view) => {
            if state.log_files_view == view {
                return;
            }

            state.log_files_view = view;

            // the tree is only fetched when it is actually shown
            if let LogRelocate::Concrete { id, .. } = &state.log_reloc {
                let id = id.clone();
                debounce_files_tree(state, ctx, id);
            }

            // list and tree select different things, so an already fetched tree takes over the
            // preview right away (a tree that still has to be fetched does it on arrival)
            if view == FilesView::Tree
                && let LogRelocate::Concrete { id, .. } = &state.log_reloc
                && state.log_files_tree_id.as_ref() == Some(id)
            {
                sync_files_tree_diff(state);
            }

            // the list always previews a file, so switching to it picks up the selected file again
            if view == FilesView::List {
                sync_files_list_diff(state);
            }
        }
        LogMsg::UpdateFilesTree { tree, id } => {
            let LogRelocate::Concrete { id: relocate, .. } = &state.log_reloc else {
                return;
            };

            // a tree fetched for a change that is not shown anymore is stale
            if relocate != &id {
                return;
            }

            state.log_files_tree_state.open_all(tree.get());

            // keep the selection of the previous tree when that node still exists
            let selected = state.log_files_tree_state.selected().to_vec();
            let visible = state.log_files_tree_state.flatten(tree.get());
            if (selected.is_empty() || !visible.iter().any(|v| v.identifier == selected))
                && let Some(first) = tree.get().first()
            {
                state
                    .log_files_tree_state
                    .select(vec![first.identifier().clone()]);
            }

            state.log_files_tree_view = tree;
            state.log_files_tree_id = Some(id);

            sync_files_tree_diff(state);
        }
        LogMsg::ScrollFilesTree(action) => {
            state.log_files_tree_state.scroll_lines(action);
            sync_files_tree_diff(state);
        }
        LogMsg::FilesTreeOpen => {
            state.log_files_tree_state.key_right();
            sync_files_tree_diff(state);
        }
        LogMsg::FilesTreeClose => {
            state.log_files_tree_state.key_left();
            sync_files_tree_diff(state);
        }
        LogMsg::UpdateFilesList(s) => {
            let LogRelocate::Concrete { id, .. } = &state.log_reloc else {
                return;
            };
            let tree_id = id.clone();

            state.log_files_list_view = s.into();
            // the diff of the shown change changed, so a fetched tree is outdated
            state.log_files_tree_id = None;

            if state.log_files_view == FilesView::List {
                sync_files_list_diff(state);
            }

            if state.log_files_view == FilesView::Tree {
                debounce_files_tree(state, ctx, tree_id);
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
                .scroll_lines(action, state.log_files_list_view.height());

            if let Some(i) = state.log_files_state.selected()
                && let Some(file) = state.log_files_list_view.lines.get(i).map(|v| {
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
            if state.log_history_state.yanking().is_some() {
                return;
            }

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
            match &state.log_modal_rebase_list {
                Some(v) if !v.lines.is_empty() => state.log_modal_rebase_list_state.0.reset(),
                _ => state.log_modal_rebase_list_state.0.select(None),
            }
        }
        LogMsg::RebaseListClose => {
            close_rebase_list(state);
        }
        LogMsg::RebaseListBack => {
            state.log_modal_rebase_from = None;
            state.log_modal_rebase_list_state.1.select(None);
        }
        LogMsg::RebaseListSelect => match state.log_modal_rebase_from.as_deref() {
            Some(from) => {
                let Some(to) = state.log_modal_rebase_list.as_ref().and_then(|view| {
                    state
                        .log_modal_rebase_list_state
                        .1
                        .selected()
                        .and_then(|i| {
                            view.lines
                                .iter()
                                .filter(|line| line.spans[0].content != from)
                                .nth(i)
                                .map(|v| v.spans[0].content.as_ref())
                        })
                }) else {
                    return;
                };

                state.log_rebase = Some(Rebase::Branch {
                    from: from.into(),
                    to: to.into(),
                });
                close_rebase_list(state);
            }
            None => {
                let Some(from) = state.log_modal_rebase_list.as_ref().and_then(|view| {
                    state
                        .log_modal_rebase_list_state
                        .0
                        .selected()
                        .and_then(|i| view.lines.get(i))
                        .map(|v| v.spans[0].content.as_ref())
                }) else {
                    return;
                };

                state.log_modal_rebase_from = Some(from.into());
                match &state.log_modal_rebase_list {
                    Some(v) if v.lines.len() > 1 => state.log_modal_rebase_list_state.1.reset(),
                    _ => state.log_modal_rebase_list_state.1.select(None),
                }
            }
        },
        LogMsg::RebaseListScroll(action) => {
            let Some(view) = state.log_modal_rebase_list.as_ref() else {
                return;
            };

            match state.log_modal_rebase_from.as_deref() {
                Some(from) => {
                    let height = view
                        .lines
                        .iter()
                        .filter(|line| line.spans[0].content != from)
                        .count();
                    state
                        .log_modal_rebase_list_state
                        .1
                        .scroll_lines(action, height);
                }
                None => {
                    state
                        .log_modal_rebase_list_state
                        .0
                        .scroll_lines(action, view.lines.len());
                }
            }
        }
        LogMsg::Rebase { id } => {
            state.log_rebase = match state.log_history_state.yanking() {
                Some(Yanking::One { id: from }) => {
                    can_rebase_onto(from, &id).then(|| Rebase::One {
                        from: from.clone(),
                        to: id,
                    })
                }
                Some(Yanking::Range {
                    base: (start, end),
                    ids,
                }) => can_rebase_onto(ids, &id).then(|| Rebase::Range {
                    start: start.clone(),
                    end: end.clone(),
                    to: id.clone(),
                }),
                None => None,
            };
        }
        LogMsg::RebaseConfirm(yes) => {
            if let Some(v) = state.log_rebase.take()
                && yes
            {
                state.log_history_state.unyank();

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
            } else if state.log_modal_rebase_list.is_some() {
                "log-rebase-list"
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

fn spawn_files_list(state: &mut MainState, ctx: &mut DefaultContext<Message, State>, id: SmolStr) {
    let jj = state.jj_handle.clone();
    ctx.queue()
        .spawn_try(async move { jj.diff_sum(&id).await.map(LogMsg::UpdateFilesList) });
}

/// Fetch the file tree of `id`, unless it is hidden or already loaded.
fn debounce_files_tree(
    state: &mut MainState,
    ctx: &mut DefaultContext<Message, State>,
    id: SmolStr,
) {
    if state.log_files_view != FilesView::Tree || state.log_files_tree_id.as_ref() == Some(&id) {
        return;
    }

    let jj = state.jj_handle.clone();
    ctx.queue().spawn_try(async move {
        jj.diff_files(&id).await.map(|raw| LogMsg::UpdateFilesTree {
            tree: PathTree::new(&raw),
            id,
        })
    });
}

/// Point the diff pane at the file selected in the tree, blank it when a directory is selected.
/// Point the diff pane at the file selected in the list, taking the first file when the selected
/// one is gone.
fn sync_files_list_diff(state: &mut MainState) {
    let LogRelocate::Concrete {
        id,
        file: file_reloc,
    } = &mut state.log_reloc
    else {
        return;
    };
    let id = id.clone();

    if let Some(file) = file_reloc
        && let Some(i) = state
            .log_files_list_view
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
            .log_files_list_view
            .lines
            .first()
            .and_then(|v| v.spans.first().map(|v| v.content.as_ref().into()));
        state.log_diff_state.reset();
    }

    let diff = file_reloc.as_ref().map(|file| (id, file.clone()));
    if let Some((id, file)) = diff {
        debounce_diff(state, id, file);
    } else {
        state.log_diff_debounce_mut().cancel();
        state.log_diff_view = Default::default();
    }
}

fn sync_files_tree_diff(state: &mut MainState) {
    let LogRelocate::Concrete { id, file } = &mut state.log_reloc else {
        return;
    };
    let Some(path) = state.log_files_tree_state.selected().last() else {
        return;
    };

    let Some(status_file) = state.log_files_tree_view.status_file(path) else {
        // a directory has no diff of its own, so the preview is blank
        *file = None;
        state.log_diff_state.reset();
        state.log_diff_view = BoxText::default();

        return;
    };
    let id = id.clone();
    // the tree keeps `ByteString`s, everything else in the log state is a `SmolStr`
    let status_file = SmolStr::new(&*status_file);

    *file = Some(status_file.clone());
    state.log_diff_state.reset();
    debounce_diff(state, id, status_file);
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
            jj.diff(&id, &status_file)
                .await
                .map(|text| LogMsg::UpdateDiff { text, version })
        });
}

fn close_rebase_list(state: &mut MainState) {
    state.log_modal_rebase_list = None;
    state.log_modal_rebase_from = None;
    state.log_modal_rebase_list_state.0.select(None);
    state.log_modal_rebase_list_state.1.select(None);
}

fn can_rebase_onto(revisions: &str, target: &str) -> bool {
    revisions
        .lines()
        .all(|revision| !revision_eq(revision, target))
}

fn revision_eq(lhs: &str, rhs: &str) -> bool {
    match (lhs.split_once('/'), rhs.split_once('/')) {
        (Some((lhs_id, lhs_offset)), Some((rhs_id, rhs_offset))) => {
            (lhs_id.starts_with(rhs_id) || rhs_id.starts_with(lhs_id)) && lhs_offset == rhs_offset
        }
        (None, None) => lhs.starts_with(rhs) || rhs.starts_with(lhs),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::{can_rebase_onto, revision_eq};

    #[test]
    fn compares_regular_revisions_by_change_id_prefix() {
        assert!(revision_eq("abcdefghijkl", "abcdefgh"));
        assert!(!revision_eq("abcdefghijkl", "xyzuvw"));
    }

    #[test]
    fn compares_divergent_revisions_by_change_id_and_offset() {
        assert!(revision_eq("abcdefghijkl/0", "abcdefgh/0"));
        assert!(!revision_eq("abcdefghijkl/0", "abcdefgh/1"));
        assert!(!revision_eq("abcdefghijkl/0", "xyzuvw/0"));
    }

    #[test]
    fn divergent_and_regular_revisions_are_different() {
        assert!(!revision_eq("abcdefghijkl/0", "xyzuvw"));
        assert!(!revision_eq("abcdefghijkl", "xyzuvw/0"));
    }

    #[test]
    fn allows_rebasing_divergent_revision_onto_regular_revision() {
        assert!(can_rebase_onto("abcdefghijkl/0", "xyzuvw"));
    }

    #[test]
    fn rejects_rebasing_onto_the_same_divergent_revision() {
        assert!(!can_rebase_onto("abcdefghijkl/0", "abcdefgh/0"));
        assert!(can_rebase_onto("abcdefghijkl/0", "abcdefgh/1"));
    }

    #[test]
    fn rejects_range_containing_the_divergent_target_revision() {
        let revisions = "abcdefghijkl/0\nxyzuvwxyzuvw\n";

        assert!(!can_rebase_onto(revisions, "abcdefgh/0"));
        assert!(can_rebase_onto(revisions, "abcdefgh/1"));
        assert!(can_rebase_onto(revisions, "mnopqrst"));
    }
}
