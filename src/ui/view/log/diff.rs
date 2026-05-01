use ratatui::{
    crossterm::event::{KeyCode, KeyModifiers},
    text::Text,
};
use ratzgo::{
    core::*,
    scroll::ScrollAction,
    text::Line,
    widget::{BorderType, ParagraphState, block, paragraph},
};

use crate::ui::{
    LogMsg,
    view::log::{LogFocus, LogLayout},
};

#[derive(Debug)]
pub struct VState<'a> {
    pub state: &'a mut ParagraphState,
    pub log_focus: &'a LogFocus,
    pub view: Text<'a>,
    pub id: Option<&'a str>,
    pub file: Option<&'a str>,
}

pub fn view<'a>(
    VState {
        state,
        log_focus,
        view,
        id,
        file,
    }: VState<'a>,
) -> impl Into<Element<'a, LogMsg>> {
    let inner = paragraph(view, state)
        .active(log_focus.is_diff())
        .on_key(
            |k| k.code == KeyCode::Esc,
            LogMsg::Layout(LogLayout::FILES_DIFF),
        )
        .on_key(
            |k| k.code == KeyCode::Char('j'),
            LogMsg::ScrollDiff(ScrollAction::Fixed(1)),
        )
        .on_key(
            |k| k.code == KeyCode::Char('k'),
            LogMsg::ScrollDiff(ScrollAction::Fixed(-1)),
        )
        .on_key(
            |k| k.code == KeyCode::Char('d') && k.modifiers == KeyModifiers::CONTROL,
            LogMsg::ScrollDiff(ScrollAction::Viewport(50)),
        )
        .on_key(
            |k| k.code == KeyCode::Char('u') && k.modifiers == KeyModifiers::CONTROL,
            LogMsg::ScrollDiff(ScrollAction::Viewport(-50)),
        )
        .on_key(
            |k| k.code == KeyCode::Char('f') && k.modifiers == KeyModifiers::CONTROL,
            LogMsg::ScrollDiff(ScrollAction::Viewport(100)),
        )
        .on_key(
            |k| k.code == KeyCode::Char('b') && k.modifiers == KeyModifiers::CONTROL,
            LogMsg::ScrollDiff(ScrollAction::Viewport(-100)),
        )
        .on_key(|k| k.code == KeyCode::Char('?'), LogMsg::Help);

    let mut v = block(inner);
    if let Some(id) = id {
        v = v.title(id);
    }
    if let Some(file) = file {
        v = v.title(Line::from(file).right_aligned());
    }
    v.bordered().border_type(BorderType::Rounded)
}
