use ratatui::widgets::Padding;
use ratzgo::{
    core::*,
    widget::{BorderType, block},
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
    pub view: &'a LogText,
}

pub fn view<'a>(VState { view, state }: VState<'a>) -> impl Into<Element<'a, BookmarksMsg>> {
    block(LogHistory::new(view, state))
        .bordered()
        .border_type(BorderType::Rounded)
        .decorate(|v| v.padding(Padding::horizontal(1)))
}
