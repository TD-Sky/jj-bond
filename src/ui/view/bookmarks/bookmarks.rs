use bytestring::ByteString;
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
        tui::{LogText, TreeText},
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
                state.bookmarks_state.select(vec![]);
                state.bookmarks_history_view = LogText::default();
                state.bookmarks_history_state.reset();
                state.bookmarks_history_debounce_mut().cancel();

                return;
            }

            let reset = match state.bookmarks_state.selected() {
                [bookmark] => state
                    .bookmarks
                    .get()
                    .iter()
                    .all(|bm| bm.identifier() != bookmark),
                [bookmark, remote] => state
                    .bookmarks
                    .get()
                    .iter()
                    .find(|v| v.identifier() == bookmark)
                    .is_none_or(|v| v.children().iter().all(|v| v.identifier() != remote)),
                [] => true,
                _ => unreachable!(),
            };

            if reset {
                state.bookmarks.get().iter().for_each(|v| {
                    state.bookmarks_state.open(vec![v.identifier().clone()]);
                });

                let bookmark = state.bookmarks.get()[0].identifier().clone();
                state.bookmarks_state.select(vec![bookmark]);
            }

            if let Some((bookmark, remote)) = selected_bookmark(state) {
                debounce_history(state, bookmark, remote);
            }
        }
        BookmarksMsg::ScrollTree(v) => {
            state.bookmarks_state.scroll_lines(v);
            if let Some((bookmark, remote)) = selected_bookmark(state) {
                state.bookmarks_history_state.reset();
                debounce_history(state, bookmark, remote);
            }
        }
        BookmarksMsg::ScrollHistory(v) => {
            state
                .bookmarks_history_state
                .scroll_vertical(&state.bookmarks_history_view, v);
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
            if state.bookmarks_history_debounce().version() != version
                && let Some((bookmark, remote)) = selected_bookmark(state)
            {
                debounce_history(state, bookmark, remote);
            } else {
                state.bookmarks_history_view = text;
            }
        }
        BookmarksMsg::ViewHistory => {
            let bookmark = match selected_bookmark(state) {
                Some((bookmark, Some(remote)))
                    if let Ok(true) = state
                        .jj_handle
                        .bookmark_remote_present(&bookmark, &remote)
                        .await =>
                {
                    format!("{bookmark}@{remote}").into()
                }
                Some((name, None)) => name,
                _ => return,
            };

            state.nav_tab = Tab::Log;

            state.log_mode = LogMode::Bookmark(bookmark);
            state.log_history_state.reset();
            state.log_reloc = LogRelocate::Index {
                index: 0,
                file: None,
            };
            state.log_layout = LogLayout::HISTORY_FILES;
            state.log_focus = state.log_layout.into();
            state.bookmarks_history_state.reset();
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
                                state.bookmarks_history_state.reset();
                            }
                            Err(e) => {
                                ratzgo::log::error("`bookmark track`", e.into_text());
                            }
                        }
                    }
                    None => {
                        let remotes_untrack =
                            match state.jj_handle.bookmark_remotes_untrack(bm).await {
                                Ok(v) => v,
                                Err(e) => {
                                    ratzgo::log::error("`bookmark remotes untrack`", e.into_text());
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
        BookmarksMsg::TrackConfirm(yes) => {
            if let Some(v) = state.bookmarks_modal_remotes.take()
                && yes
                && let Some(i) = state.bookmarks_modal_remotes_state.selected()
                && let Some(remote) = v.remotes.get(i)
            {
                match state.jj_handle.bookmark_track(&v.bookmark, remote).await {
                    Ok(_) => {
                        state
                            .bookmarks_state
                            .select(vec![v.bookmark, remote.as_str().into()]);
                        state.bookmarks_history_state.reset();
                    }
                    Err(e) => {
                        ratzgo::log::error("`bookmark track`", e.into_text());
                    }
                }
            }
        }
        BookmarksMsg::Untrack => {
            if let [name, remote] = state.bookmarks_state.selected()
                && remote != "git"
            {
                match state.jj_handle.bookmark_untrack(name, remote).await {
                    Ok(_) => {
                        let bookmark = format!("{name}@{remote}");
                        state.bookmarks_state.select(vec![bookmark.into()]);
                        state.bookmarks_history_state.reset();
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
            {
                match state.jj_handle.bookmark_delete(&name).await {
                    Ok(_) => state.bookmarks_history_state.reset(),
                    Err(e) => ratzgo::log::error("`bookmark delete`", e.into_text()),
                }
            }
        }
        BookmarksMsg::ScrollRemotes(action) => {
            if let Some(v) = &state.bookmarks_modal_remotes {
                state
                    .bookmarks_modal_remotes_state
                    .scroll_lines(action, v.remotes.len());
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
            .map(|v| BookmarksMsg::UpdateTree(TreeText::new(v)))
    });
}

fn debounce_history(state: &mut MainState, bookmark: ByteString, remote: Option<ByteString>) {
    let jj = state.jj_handle.clone();
    state
        .bookmarks_history_debounce_mut()
        .spawn_try(|version| async move {
            let bookmark = match remote {
                Some(remote)
                    if let Ok(true) = jj.bookmark_remote_present(&bookmark, &remote).await =>
                {
                    format!("{bookmark}@{remote}").into()
                }
                None => bookmark,
                _ => {
                    return Ok(BookmarksMsg::UpdateHistory {
                        text: LogText::default(),
                        version,
                    });
                }
            };

            jj.log(&LogMode::Bookmark(bookmark))
                .await
                .map(|v| BookmarksMsg::UpdateHistory {
                    text: LogText::new(v),
                    version,
                })
        });
}

fn selected_bookmark(state: &MainState) -> Option<(ByteString, Option<ByteString>)> {
    match state.bookmarks_state.selected() {
        [bookmark] => Some((bookmark.clone(), None)),
        [bookmark, remote] => Some((bookmark.clone(), Some(remote.clone()))),
        _ => None,
    }
}
