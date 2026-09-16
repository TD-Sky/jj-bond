use std::{cell::Cell, rc::Rc};

use ratatui::{
    layout::{Constraint, Rect},
    widgets::Padding,
};
use ratzgo::{
    core::*,
    widget::{BorderType, ScrollbarParams, block, scrollbar},
};

use crate::{
    ui::{
        BookmarksMsg,
        widgets::{LogHistory, LogHistoryState},
    },
    utils::tui::LogText,
};

#[derive(Debug)]
pub struct VState<'a> {
    pub state: &'a mut LogHistoryState,
    pub area: &'a Rc<Cell<Rect>>,
    pub view: &'a LogText,
}

pub fn view<'a>(VState { state, area, view }: VState<'a>) -> impl Into<Element<'a, BookmarksMsg>> {
    let offset = state.offset();
    let height = view.text().height();

    block(LogHistory::new(view, state).bind_area(area))
        .bordered()
        .border_type(BorderType::Rounded)
        .decorate(|v| v.padding(Padding::horizontal(1)))
        .widget_right_opt(
            scrollbar(ScrollbarParams {
                content_length: height,
                viewport: Area::Ref(area.clone()),
                position: offset,
            }),
            {
                let viewport = area.clone();
                move |area| {
                    (height > viewport.get().height as usize)
                        .then(|| area.centered_vertically(Constraint::Percentage(90)))
                }
            },
        )
}
