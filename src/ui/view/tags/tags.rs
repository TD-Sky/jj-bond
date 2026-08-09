use ratatui::macros::constraints;
use ratzgo::{core::*, event::DefaultContext, widget::row};

use crate::{
    ui::{
        HelpMsg, LogRelocate, MainState, Message, State, TagRelocate, TagsMsg,
        view::{
            log::LogLayout,
            nav::Tab,
            tags::{history, list},
        },
    },
    utils::{jj::LogMode, tui::LogText},
};

pub fn view<'a>(state: &'a mut MainState) -> Element<'a, TagsMsg> {
    let css = constraints![==1/3, ==2/3];

    row! {
        css;
        [
            list::view(list::VState {
                view: state.tags.get(),
                state: &mut state.tags_state,
                mount_point: &state.mount_point,
                modal_delete_tag: state.tags_modal_delete_tag.as_deref(),
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
        TagsMsg::UpdateList(v) => {
            state.tags = v.into();
            state.tags.lines.dedup_by(|lhs, rhs| {
                lhs.spans[0].content.as_ref() == rhs.spans[0].content.as_ref()
            });

            let pair = match &state.tags_reloc {
                TagRelocate::Name(tag) => {
                    state.tags.lines.iter().enumerate().find_map(|(i, line)| {
                        line.spans
                            .first()
                            .filter(|v| v.content == tag.as_str())
                            .map(|v| (i, v.content.as_ref()))
                    })
                }
                TagRelocate::Index(index) => match *index < state.tags.height() {
                    true => state.tags.lines[*index]
                        .spans
                        .first()
                        .map(|v| (*index, v.content.as_ref())),
                    false => state
                        .tags
                        .lines
                        .first()
                        .and_then(|v| v.spans.first().map(|v| (0, v.content.as_ref()))),
                },
            };

            let Some((index, tag)) = pair else {
                state.tags_state.select(None);
                state.tags_reloc = TagRelocate::default();
                state.tags_history_view = LogText::default();
                state.tags_history_state.reset();
                state.tags_history_debounce_mut().cancel();

                return;
            };

            state.tags_state.select(Some(index));
            state.tags_reloc = TagRelocate::Name(tag.into());

            debounce_history(state, LogMode::Tag(tag.into()));
        }
        TagsMsg::ScrollList(v) => {
            state.tags_state.scroll_vertical(v, state.tags.height());

            if let Some(index) = state.tags_state.selected()
                && let Some(tag) = state
                    .tags
                    .lines
                    .get(index)
                    .map(|v| v.spans[0].content.as_ref())
            {
                state.tags_reloc = TagRelocate::Name(tag.into());

                debounce_history(state, LogMode::Tag(tag.into()));
            }
        }
        TagsMsg::ScrollHistory(v) => {
            state
                .tags_history_state
                .scroll_vertical(&state.tags_history_view, v);
        }
        TagsMsg::UpdateHistory { text, version } => {
            if state.tags_history_debounce().version() != version
                && let TagRelocate::Name(tag) = &state.tags_reloc
            {
                debounce_history(state, LogMode::Tag(tag.clone()));
            } else {
                state.tags_history_view = text;
                state.tags_history_state.reset();
            }
        }
        TagsMsg::ViewHistory => {
            if let TagRelocate::Name(tag) = &state.tags_reloc {
                state.nav_tab = Tab::Log;

                state.log_mode = LogMode::Tag(tag.clone());
                state.log_history_state.reset();
                state.log_reloc = LogRelocate::Index {
                    index: 0,
                    file: None,
                };
                state.log_layout = LogLayout::HISTORY_FILES;
                state.log_focus = state.log_layout.into();
                ctx.queue().push(Message::Refresh);
            }
        }
        TagsMsg::Delete => {
            if let TagRelocate::Name(tag) = &state.tags_reloc {
                state.tags_modal_delete_tag = Some(tag.clone());
            }
        }
        TagsMsg::DeleteConfirm(yes) => {
            if let Some(tag) = state.tags_modal_delete_tag.take()
                && yes
            {
                match state.jj_handle.tag_delete(&tag).await {
                    Ok(_) => {
                        state.tags_reloc =
                            TagRelocate::Index(state.tags_state.selected().unwrap_or_default());
                    }
                    Err(e) => {
                        ratzgo::log::error("`tag delete`", e.into_text());
                    }
                }
            }
        }
        TagsMsg::Help => {
            let page = if state.tags_modal_delete_tag.is_some() {
                "confirm-modal"
            } else {
                "tags-list"
            };

            ctx.queue().push(HelpMsg::Page(page));
        }
    }
}

pub fn refresh(state: &mut MainState, ctx: &mut DefaultContext<Message, State>) {
    let jj_handle = state.jj_handle.clone();
    ctx.queue()
        .spawn_try(async move { jj_handle.tags().await.map(TagsMsg::UpdateList) });
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
