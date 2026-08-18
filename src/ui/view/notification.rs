use std::collections::VecDeque;

use ratatui::{crossterm::event::KeyCode, macros::constraint};
use ratzgo::{core::*, widget::MountPoint};

use crate::ui::{Message, NotifyMsg, widgets::Notification};

#[derive(Debug, Default)]
pub struct State {
    pub events: VecDeque<ratzgo::log::Event>,
    pub mount_point: MountPoint<Message>,
}

pub fn view<'a>(
    State {
        events,
        mount_point,
    }: &'a State,
) -> impl Into<Element<'a, Message>> {
    if let Some(event) = events.back() {
        mount_point.mount(
            Notification::new(
                format!("{} : {}", event.level, event.target),
                event.text.clone(),
            )
            .on_key(|key| key.code == KeyCode::Esc, NotifyMsg::Confirm.into()),
            |area| area.centered(constraint!(==1/2), constraint!(==3/5)),
        );
    }

    mount_point.view()
}

pub fn update(state: &mut State, msg: NotifyMsg) {
    match msg {
        NotifyMsg::LogEvent(event) => {
            state.events.push_front(event);
        }
        NotifyMsg::Confirm => {
            state.events.pop_back();
        }
    }
}
