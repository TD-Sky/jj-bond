use bytestring::ByteString;
use ratatui::macros::constraints;
use ratzgo::{core::*, event::DefaultContext, widget::row};

use crate::{
    ui::{
        HelpMsg, LogRelocate, MainState, Message, State, TagPush, TagTrack, TagsMsg,
        view::{
            log::LogLayout,
            nav::Tab,
            tags::{history, tree},
        },
    },
    utils::{
        jj::LogMode,
        tui::{LogText, TreeText},
    },
};

pub fn view<'a>(state: &'a mut MainState) -> Element<'a, TagsMsg> {
    let css = constraints![==1/3, ==2/3];

    row! {
        css;
        [
            tree::view(tree::VState {
                view: &state.tags,
                state: &mut state.tags_state,
                mount_point: &state.mount_point,
                modal_delete: state.tags_modal_delete.as_deref(),
                modal_push: state.tags_modal_push.as_ref(),
                modal_remotes: state
                    .tags_modal_remotes
                    .as_ref()
                    .map(|v| (v.remotes.as_slice(), &mut state.tags_modal_remotes_state)),
            }),
            history::view(history::VState {
                view: &state.tags_history_view,
                state: &mut state.tags_history_state,
            })
        ]
    }
    .into()
}

pub async fn update(state: &mut MainState, msg: TagsMsg, ctx: &mut DefaultContext<Message, State>) {
    match msg {
        TagsMsg::UpdateTree(v) => {
            state.tags = v;

            if state.tags.get().is_empty() {
                state.tags_state.select(vec![]);
                state.tags_history_view = LogText::default();
                state.tags_history_state.reset();
                state.tags_history_debounce_mut().cancel();

                return;
            }

            let reset = match state.tags_state.selected() {
                [tag] => state.tags.get().iter().all(|v| v.identifier() != tag),
                [tag, remote] => state
                    .tags
                    .get()
                    .iter()
                    .find(|v| v.identifier() == tag)
                    .is_none_or(|v| v.children().iter().all(|v| v.identifier() != remote)),
                [] => true,
                _ => unreachable!(),
            };

            if reset {
                state.tags.get().iter().for_each(|v| {
                    state.tags_state.open(vec![v.identifier().clone()]);
                });

                let tag = state.tags.get()[0].identifier().clone();
                state.tags_state.select(vec![tag.clone()]);

                debounce_history(state, LogMode::Tag(tag));
            }
        }
        TagsMsg::ScrollTree(v) => {
            state.tags_state.scroll_vertical(v);
            if let Some(tag) = selected_tag(state) {
                debounce_history(state, LogMode::Tag(tag));
            }
        }
        TagsMsg::ScrollHistory(v) => {
            state
                .tags_history_state
                .scroll_vertical(&state.tags_history_view, v);
        }
        TagsMsg::TagOpen => {
            let path = state.tags_state.selected().to_vec();
            state.tags_state.open(path);
        }
        TagsMsg::TagClose => {
            let path = state.tags_state.selected().to_vec();
            state.tags_state.close(&path);
        }
        TagsMsg::UpdateHistory { text, version } => {
            if state.tags_history_debounce().version() != version
                && let Some(tag) = selected_tag(state)
            {
                debounce_history(state, LogMode::Tag(tag));
            } else {
                state.tags_history_view = text;
                state.tags_history_state.reset();
            }
        }
        TagsMsg::ViewHistory => {
            let Some(tag) = selected_tag(state) else {
                return;
            };

            state.nav_tab = Tab::Log;

            state.log_mode = LogMode::Tag(tag);
            state.log_history_state.reset();
            state.log_reloc = LogRelocate::Index {
                index: 0,
                file: None,
            };
            state.log_layout = LogLayout::HISTORY_FILES;
            state.log_focus = state.log_layout.into();
            ctx.queue().push(Message::Refresh);
        }
        TagsMsg::Delete => {
            if let [name] = state.tags_state.selected()
                && !name.contains('@')
            {
                state.tags_modal_delete = Some(name.clone());
            }
        }
        TagsMsg::DeleteConfirm(yes) => {
            if let Some(tag) = state.tags_modal_delete.take()
                && yes
                && let Err(e) = state.jj_handle.tag_delete(&tag).await
            {
                ratzgo::log::error("`tag delete`", e.into_text());
            }
        }
        TagsMsg::Track => {
            if let [tag] = state.tags_state.selected() {
                match tag.split_once('@') {
                    Some((name, remote)) => match state.jj_handle.tag_track(name, remote).await {
                        Ok(_) => {
                            let name = tag.slice_ref(name);

                            state.tags_state.select(vec![name]);
                        }
                        Err(e) => {
                            ratzgo::log::error("`tag track`", e.into_text());
                        }
                    },
                    None => {
                        let remotes_untrack = match state.jj_handle.tag_remotes_untrack(tag).await {
                            Ok(v) => v,
                            Err(e) => {
                                ratzgo::log::error("`tag remotes untrack`", e.into_text());
                                return;
                            }
                        };
                        if remotes_untrack.is_empty() {
                            return;
                        }
                        state.tags_modal_remotes = Some(TagTrack {
                            tag: tag.clone(),
                            remotes: remotes_untrack,
                        });
                        state.tags_modal_remotes_state.reset();
                    }
                }
            }
        }
        TagsMsg::TrackConfirm(yes) => {
            if let Some(v) = state.tags_modal_remotes.take()
                && yes
                && let Some(i) = state.tags_modal_remotes_state.selected()
                && let Some(remote) = v.remotes.get(i)
            {
                match state.jj_handle.tag_track(&v.tag, remote).await {
                    Ok(_) => {
                        state.tags_state.select(vec![v.tag, remote.as_str().into()]);
                    }
                    Err(e) => {
                        ratzgo::log::error("`tag track`", e.into_text());
                    }
                }
            }
        }
        TagsMsg::ScrollRemotes(action) => {
            if let Some(v) = &state.tags_modal_remotes {
                state
                    .tags_modal_remotes_state
                    .scroll_vertical(action, v.remotes.len());
            }
        }
        TagsMsg::Untrack => {
            if let [name, remote] = state.tags_state.selected()
                && remote != "git"
            {
                match state.jj_handle.tag_untrack(name, remote).await {
                    Ok(_) => {
                        let tag = format!("{name}@{remote}");
                        state.tags_state.select(vec![tag.into()]);
                    }
                    Err(e) => {
                        ratzgo::log::error("`tag untrack`", e.into_text());
                    }
                }
            }
        }
        TagsMsg::Push => {
            if let [name, remote] = state.tags_state.selected()
                && remote != "git"
                && let Ok(false) = state.jj_handle.tag_synced_remote(name, remote).await
            {
                state.tags_modal_push = Some(TagPush {
                    name: name.clone(),
                    remote: remote.clone(),
                });
            }
        }
        TagsMsg::PushConfirm(yes) => {
            if let Some(v) = state.tags_modal_push.take()
                && yes
            {
                match state.jj_handle.push_tag(&v.name, &v.remote).await {
                    Ok(_) => {
                        state.tags_state.select(vec![v.name, v.remote]);
                    }
                    Err(e) => ratzgo::log::error("`push tag`", e.into_text()),
                }
            }
        }
        TagsMsg::Help => {
            let page = if state.tags_modal_delete.is_some() || state.tags_modal_push.is_some() {
                "confirm-modal"
            } else if state.tags_modal_remotes.is_some() {
                "tags-remotes"
            } else {
                "tags-tree"
            };

            ctx.queue().push(HelpMsg::Page(page));
        }
    }
}

pub fn refresh(state: &mut MainState, ctx: &mut DefaultContext<Message, State>) {
    let jj_handle = state.jj_handle.clone();
    ctx.queue().spawn_try(async move {
        jj_handle
            .tag_tree()
            .await
            .map(|s| TagsMsg::UpdateTree(TreeText::new(s)))
    });
}

fn debounce_history(state: &mut MainState, mode: LogMode) {
    let jj = state.jj_handle.clone();
    state
        .tags_history_debounce_mut()
        .spawn_try(|version| async move {
            jj.log(&mode).await.map(|v| TagsMsg::UpdateHistory {
                text: LogText::new(v),
                version,
            })
        });
}

fn selected_tag(state: &MainState) -> Option<ByteString> {
    match state.tags_state.selected() {
        [tag] => Some(tag.clone()),
        [tag, remote] => Some(format!("{tag}@{remote}").into()),
        _ => None,
    }
}
