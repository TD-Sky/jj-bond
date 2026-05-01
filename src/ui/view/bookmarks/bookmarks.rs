use ratatui::macros::constraints;
use ratzgo::{core::*, event::DefaultContext, widget::row};

use crate::{
    ui::{
        BookmarkTrack, BookmarksMsg, HelpMsg, LogRelocate, MainState, Message, State,
        view::{
            bookmarks::{history, tree},
            log::LogLayout,
            nav::Tab,
        },
    },
    utils::{
        jj::LogMode,
        tui::{BookmarkTree, LogText},
    },
};

pub fn view<'a>(state: &'a mut MainState) -> Element<'a, BookmarksMsg> {
    let css = constraints![==1/3, ==2/3];

    row! {
        css;
        [
            tree::view(tree::VState {
                view: &state.bookmarks,
                state: &mut state.bookmarks_state,
                mount_point: &state.mount_point,
                modal_delete: state.bookmarks_modal_delete.as_ref(),
                modal_remotes: state
                    .bookmarks_modal_remotes
                    .as_ref()
                    .map(|v| (v.remotes.as_slice(), &mut state.bookmarks_modal_remotes_state)),
            }),
            history::view(history::VState {
                view: &state.bookmarks_history_view,
                state: &mut state.bookmarks_history_state,
            })
        ]
    }
    .into()
}

pub async fn update(
    state: &mut MainState,
    msg: BookmarksMsg,
    ctx: &mut DefaultContext<Message, State>,
) {
    match msg {
        BookmarksMsg::UpdateTree(v) => {
            state.bookmarks = v;

            if state.bookmarks.get().is_empty() {
                state.bookmarks_history_view = LogText::default();
                state.bookmarks_history_debounce_mut().cancel();
            }

            let reset = match state.bookmarks_state.selected() {
                [bookmark] => state
                    .bookmarks
                    .get()
                    .iter()
                    .all(|bm| bm.identifier() != bookmark),
                [bookmark, remote] => state.bookmarks.get().iter().all(|bm| {
                    bm.identifier() != bookmark
                        && bm.children().iter().all(|rm| rm.identifier() != remote)
                }),
                [] => true,
                _ => unreachable!(),
            };

            if reset {
                state.bookmarks.get().iter().for_each(|v| {
                    state.bookmarks_state.open(vec![v.identifier().clone()]);
                });

                let Some(bookmark) = state
                    .bookmarks
                    .get()
                    .first()
                    .map(|v| v.identifier().clone())
                else {
                    return;
                };
                state.bookmarks_state.select(vec![bookmark.clone()]);

                debounce_history(state, LogMode::Bookmark(bookmark));
            }
        }
        BookmarksMsg::ScrollTree(v) => {
            state.bookmarks_state.scroll_vertical(v);
            let bookmark = match state.bookmarks_state.selected() {
                [bm] => bm.clone(),
                [bm, rm] => format!("{bm}{rm}").into(),
                _ => {
                    return;
                }
            };

            debounce_history(state, LogMode::Bookmark(bookmark));
        }
        BookmarksMsg::ScrollHistory(v) => {
            state
                .bookmarks_history_state
                .scroll_vertical(&state.log_history, v);
        }
        BookmarksMsg::BookmarkOpen => {
            let path = state.bookmarks_state.selected().to_vec();
            state.bookmarks_state.open(path);
        }
        BookmarksMsg::BookmarkClose => {
            let path = state.bookmarks_state.selected().to_vec();
            state.bookmarks_state.close(&path);
        }
        BookmarksMsg::UpdateHistory { text, version } => {
            if state.bookmarks_history_debounce().version() != version {
                let bookmark = match state.bookmarks_state.selected() {
                    [bm] => bm.clone(),
                    [bm, rm] => format!("{bm}{rm}").into(),
                    _ => return,
                };
                debounce_history(state, LogMode::Bookmark(bookmark));
            } else {
                state.bookmarks_history_view = text;
                state.bookmarks_history_state.reset();
            }
        }
        BookmarksMsg::ViewHistory => {
            state.nav_tab = Tab::Log;

            let bookmark = match state.bookmarks_state.selected() {
                [bm] => bm.clone(),
                [bm, rm] => format!("{bm}{rm}").into(),
                _ => {
                    return;
                }
            };
            state.log_mode = LogMode::Bookmark(bookmark);
            state.log_reloc = LogRelocate::Index {
                index: 0,
                file: None,
            };
            state.log_layout = LogLayout::HISTORY_FILES;
            state.log_focus = state.log_layout.into();
            ctx.queue().push(Message::Refresh);
        }
        BookmarksMsg::Track => {
            if let [bm] = state.bookmarks_state.selected() {
                match bm.split_once('@') {
                    Some((name, remote)) => {
                        match state.jj_handle.bookmark_track(name, remote).await {
                            Ok(_) => {
                                let name = bm.slice_ref(name);

                                state.bookmarks_state.select(vec![name]);
                            }
                            Err(e) => {
                                ratzgo::log::error("`bookmark track`", e.into_text());
                            }
                        }
                    }
                    None => {
                        let remotes_untrack = match state.jj_handle.remotes_untrack(bm).await {
                            Ok(v) => v,
                            Err(e) => {
                                ratzgo::log::error("remote_untracks", e.into_text());
                                return;
                            }
                        };
                        if remotes_untrack.is_empty() {
                            return;
                        }
                        state.bookmarks_modal_remotes = Some(BookmarkTrack {
                            bookmark: bm.clone(),
                            remotes: remotes_untrack,
                        });
                        state.bookmarks_modal_remotes_state.reset();
                    }
                }
            }
        }
        BookmarksMsg::Untrack => {
            if let [name, remote] = state.bookmarks_state.selected()
                && let Some(remote) = remote.strip_prefix('@')
                && remote != "git"
            {
                match state.jj_handle.bookmark_untrack(name, remote).await {
                    Ok(_) => {
                        let bookmark = format!("{name}@{remote}").into();
                        state.bookmarks_state.select(vec![bookmark]);
                    }
                    Err(e) => {
                        ratzgo::log::error("`bookmark untrack`", e.into_text());
                    }
                }
            }
        }

        BookmarksMsg::Delete => {
            if let [name] = state.bookmarks_state.selected()
                && !name.contains('@')
            {
                state.bookmarks_modal_delete = Some(name.clone());
            }
        }
        BookmarksMsg::DeleteConfirm(yes) => {
            if let Some(name) = state.bookmarks_modal_delete.take()
                && yes
                && let Err(e) = state.jj_handle.bookmark_delete(&name).await
            {
                ratzgo::log::error("`bookmark delete`", e.into_text());
            }
        }
        BookmarksMsg::ScrollRemotes(action) => {
            if let Some(v) = &state.bookmarks_modal_remotes {
                state
                    .bookmarks_modal_remotes_state
                    .scroll_vertical(action, v.remotes.len());
            }
        }
        BookmarksMsg::TrackConfirm(yes) => {
            if let Some(v) = state.bookmarks_modal_remotes.take()
                && yes
                && let Some(i) = state.bookmarks_modal_remotes_state.selected()
                && let Some(remote) = v.remotes.get(i)
            {
                match state.jj_handle.bookmark_track(&v.bookmark, remote).await {
                    Ok(_) => {
                        state.bookmarks_state.select(vec![v.bookmark]);
                    }
                    Err(e) => {
                        ratzgo::log::error("`bookmark track`", e.into_text());
                    }
                }
            }
        }
        BookmarksMsg::Help => {
            let page = if state.bookmarks_modal_delete.is_some() {
                "confirm-modal"
            } else if state.bookmarks_modal_remotes.is_some() {
                "bookmarks-remotes"
            } else {
                "bookmarks-tree"
            };

            ctx.queue().push(HelpMsg::Page(page));
        }
    }
}

pub fn refresh(state: &mut MainState, ctx: &mut DefaultContext<Message, State>) {
    let jj_handle = state.jj_handle.clone();
    ctx.queue().spawn_try(async move {
        jj_handle
            .bookmark_tree()
            .await
            .map(|v| BookmarksMsg::UpdateTree(BookmarkTree::new(v)))
    });
}

fn debounce_history(state: &mut MainState, mode: LogMode) {
    let jj = state.jj_handle.clone();
    state
        .bookmarks_history_debounce_mut()
        .spawn_try(|version| async move {
            jj.log(&mode).await.map(|v| BookmarksMsg::UpdateHistory {
                text: LogText::new(v),
                version,
            })
        });
}
