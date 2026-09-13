use std::{cell::Cell, rc::Rc};

use ratatui::{
    crossterm::event::{KeyCode, KeyModifiers},
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    text::Text,
};
use ratzgo::{
    core::*,
    scroll::ScrollAction,
    text::Line,
    widget::{BorderType, ListState, ScrollbarParams, block, list, scrollbar},
};

use crate::ui::{
    LogMsg,
    view::log::{LogFocus, LogLayout},
};

#[derive(Debug)]
pub struct VState<'a> {
    pub state: &'a mut ListState,
    pub area: &'a Rc<Cell<Rect>>,
    pub log_focus: &'a LogFocus,
    pub view: Text<'a>,
    pub id: Option<&'a str>,
}

pub fn view<'a>(
    VState {
        state,
        area,
        log_focus,
        view,
        id,
    }: VState<'a>,
) -> impl Into<Element<'a, LogMsg>> {
    let offset = state.offset();
    let height = view.height();

    let inner = list(state)
        .items(view)
        .bind_area(area)
        .active(log_focus.is_files())
        .decorate(|v| v.highlight_style(Style::new().add_modifier(Modifier::REVERSED)))
        .on_key(
            |k| k.code == KeyCode::Char('j'),
            LogMsg::ScrollFiles(ScrollAction::Fixed(1)),
        )
        .on_key(
            |k| k.code == KeyCode::Char('k'),
            LogMsg::ScrollFiles(ScrollAction::Fixed(-1)),
        )
        .on_key(
            |k| k.code == KeyCode::Char('d') && k.modifiers == KeyModifiers::CONTROL,
            LogMsg::ScrollFiles(ScrollAction::Viewport(50)),
        )
        .on_key(
            |k| k.code == KeyCode::Char('u') && k.modifiers == KeyModifiers::CONTROL,
            LogMsg::ScrollFiles(ScrollAction::Viewport(-50)),
        )
        .on_key(
            |k| k.code == KeyCode::Char('f') && k.modifiers == KeyModifiers::CONTROL,
            LogMsg::ScrollFiles(ScrollAction::Viewport(100)),
        )
        .on_key(
            |k| k.code == KeyCode::Char('b') && k.modifiers == KeyModifiers::CONTROL,
            LogMsg::ScrollFiles(ScrollAction::Viewport(-100)),
        )
        .on_key(
            |k| k.code == KeyCode::Enter,
            LogMsg::Layout(LogLayout::DIFF),
        )
        .on_key(
            |k| k.code == KeyCode::Esc,
            LogMsg::Layout(LogLayout::HISTORY_FILES),
        )
        .on_key(|k| k.code == KeyCode::Char('?'), LogMsg::Help);

    let mut v = block(inner);
    if let Some(id) = id {
        v = v.title(Line::from(id).style(Style::default().fg(Color::Indexed(13))));
    }

    v.bordered()
        .border_type(BorderType::Rounded)
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
