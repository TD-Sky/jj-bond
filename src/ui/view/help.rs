use std::{cell::Cell, rc::Rc};

use ratatui::{layout::Rect, macros::constraint};
use ratzgo::{
    core::*,
    widget::{MountPoint, TableState},
};

use crate::ui::{
    HelpMsg, Message,
    widgets::{keymap, keymap_at},
};

#[derive(Debug, Default)]
pub struct State {
    pub page: Option<&'static str>,
    pub state: TableState,
    pub area: Rc<Cell<Rect>>,
    pub mount_point: MountPoint<Message>,
}

pub fn view<'a>(
    State {
        page,
        state,
        area,
        mount_point,
    }: &'a mut State,
) -> Element<'a, Message> {
    if let Some(page) = page {
        mount_point.mount(keymap(state, page, area).into().map(Into::into), |area| {
            area.centered(constraint!(==50%), constraint!(==70%))
        });
    }

    mount_point.view().into()
}

pub fn update(state: &mut State, msg: HelpMsg) {
    match msg {
        HelpMsg::Page(page) => {
            state.page = Some(page);
            state.state.reset();
        }
        HelpMsg::Scroll(action) => {
            if let Some(page) = state.page {
                state.state.scroll_lines(action, keymap_at(page).len());
            }
        }
        HelpMsg::Close => {
            state.page.take();
        }
    }
}
