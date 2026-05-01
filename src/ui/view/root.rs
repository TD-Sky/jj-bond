use std::time::Duration;

use futures_util::StreamExt;
use ratatui::{crossterm::event::KeyCode, macros::constraints};
use ratzgo::{
    core::*,
    event::DefaultContext,
    widget::{column, stack},
};

use crate::{
    ui::{
        LogMsg, Message, NavMsg, NotifyMsg, State,
        view::{
            bookmarks, help, hint, log,
            nav::{self, Tab},
            notification, operations, tags,
        },
    },
    utils::jj::{JJHandle, NotifyGitChange},
};

pub async fn init(state: &mut State, ctx: &mut DefaultContext<Message, State>) {
    state.main.jj_handle = match JJHandle::current() {
        Ok(v) => v,
        Err(e) => {
            tracing::error!(error = %e, "get current jj workspace failed");
            ctx.exit(true);
            return;
        }
    };

    let debounce_duration = Duration::from_millis(50);
    state
        .main
        .log_show_debounce
        .set(ctx.make_debounce(debounce_duration))
        .expect("log show debounce must only be initialized once");
    state
        .main
        .log_diff_debounce
        .set(ctx.make_debounce(debounce_duration))
        .expect("log diff debounce must only be initialized once");
    state
        .main
        .bookmarks_history_debounce
        .set(ctx.make_debounce(debounce_duration))
        .expect("bookmarks history debounce must only be initialized once");
    state
        .main
        .tags_history_debounce
        .set(ctx.make_debounce(debounce_duration))
        .expect("tags history debounce must only be initialized once");

    let log_stream = ratzgo::log::init();

    match NotifyGitChange::new(state.main.jj_handle.clone()) {
        Ok(notify) => {
            ctx.select_mut()
                .source(notify.into_stream().map(|_| Message::Refresh));
        }
        Err(e) => {
            tracing::error!(error = %e, "watch .jj/working_copy failed");
            ctx.exit(true);
            return;
        }
    }
    ctx.select_mut()
        .source(log_stream.map(|ev| Message::Notify(NotifyMsg::LogEvent(ev))));

    if let Err(e) = state.main.jj_handle.workspace_update_stale().await {
        ratzgo::log::error("workspace update-stale", e.into_text());
    }

    ctx.queue().push(LogMsg::Fetch);
    ctx.queue().push(Message::Refresh);
}

pub fn view(state: &mut State) -> Element<'_, Message> {
    let hint_vstate = hint::State {
        fetching: state.main.log_fetching.clone(),
        pushing: state.main.log_pushing.clone(),
    };

    let main_mp = state.main.mount_point.view();
    stack![
        column! [
            constraints![==3, ==100%, ==3];
            [
                nav::view(state.main.nav_tab).into().map(Into::into),
                match state.main.nav_tab {
                    Tab::Log => log::view(&mut state.main).map(Into::into),
                    Tab::Bookmarks => bookmarks::view(&mut state.main).map(Into::into),
                    Tab::Tags => tags::view(&mut state.main).map(Into::into),
                    Tab::Operations => operations::view(&mut state.main).map(Into::into),
                },
                hint::view(hint_vstate)
            ]
        ]
        .on_key(
            |k| k.code == KeyCode::Char('1'),
            NavMsg::TabSelect(Tab::Log).into(),
        )
        .on_key(
            |k| k.code == KeyCode::Char('2'),
            NavMsg::TabSelect(Tab::Bookmarks).into(),
        )
        .on_key(
            |k| k.code == KeyCode::Char('3'),
            NavMsg::TabSelect(Tab::Tags).into(),
        )
        .on_key(
            |k| k.code == KeyCode::Char('4'),
            NavMsg::TabSelect(Tab::Operations).into(),
        )
        .on_key(|k| k.code == KeyCode::Char('H'), NavMsg::TabPrev.into())
        .on_key(|k| k.code == KeyCode::Char('L'), NavMsg::TabNext.into())
        .on_key(|k| k.code == KeyCode::Char('q'), Message::Exit)
        .active(true),
        main_mp,
        help::view(&mut state.help),
        notification::view(&state.notify),
    ]
    .into()
}

pub async fn update(state: &mut State, msg: Message, ctx: &mut DefaultContext<Message, State>) {
    match msg {
        Message::Nav(msg) => {
            nav::update(&mut state.main, msg, ctx);
        }
        Message::Log(msg) => {
            log::update(&mut state.main, msg, ctx).await;
        }
        Message::Bookmarks(msg) => {
            bookmarks::update(&mut state.main, msg, ctx).await;
        }
        Message::Tags(msg) => {
            tags::update(&mut state.main, msg, ctx).await;
        }
        Message::Op(msg) => {
            operations::update(&mut state.main, msg, ctx);
        }
        Message::Notify(msg) => notification::update(&mut state.notify, msg),
        Message::Help(msg) => help::update(&mut state.help, msg),
        Message::Refresh => match state.main.nav_tab {
            Tab::Log => log::refresh(&mut state.main, ctx),
            Tab::Bookmarks => bookmarks::refresh(&mut state.main, ctx),
            Tab::Tags => tags::refresh(&mut state.main, ctx),
            Tab::Operations => operations::refresh(&mut state.main, ctx),
        },
        Message::Exit => ctx.exit(true),
    }
}
