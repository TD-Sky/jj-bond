use ratatui::{
    crossterm::event::{KeyCode, KeyModifiers},
    macros::constraints,
    prelude::*,
};
use ratzgo::{
    core::*,
    scroll::ScrollAction,
    widget::{BorderType, ListState, block, column, list},
};

use crate::ui::{
    LogMsg, Message,
    widgets::{TextArea, TextAreaState},
};

#[derive(Debug)]
pub struct VState<'a> {
    pub view: &'a Text<'static>,
    pub state: &'a mut ListState,
    pub input: Option<&'a mut TextAreaState>,
}

pub fn view<'a>(VState { view, state, input }: VState<'a>) -> Element<'a, Message> {
    let inner: Element<LogMsg> = match input {
        Some(input) => {
            let input = TextArea::new(input)
                .active(true)
                .on_key_with(|k| Some(LogMsg::CreatingBookmark { key: *k }));
            column! [
                constraints![==1, ==100%];
                [
                    input,
                    list(state).items(view.clone()),
                ]
            ]
            .into()
        }
        None => {
            let mut view = view.clone();
            view.lines.insert(
                0,
                Line::styled("Create bookmark", Style::default().fg(Color::Yellow)),
            );

            list(state)
                .items(view)
                .decorate(|v| v.highlight_style(Style::default().add_modifier(Modifier::REVERSED)))
                .on_key(
                    |k| k.code == KeyCode::Char('k'),
                    LogMsg::BookmarkListScroll(ScrollAction::Fixed(-1)),
                )
                .on_key(
                    |k| k.code == KeyCode::Char('j'),
                    LogMsg::BookmarkListScroll(ScrollAction::Fixed(1)),
                )
                .on_key(
                    |k| k.code == KeyCode::Char('d') && k.modifiers == KeyModifiers::CONTROL,
                    LogMsg::BookmarkListScroll(ScrollAction::Viewport(50)),
                )
                .on_key(
                    |k| k.code == KeyCode::Char('u') && k.modifiers == KeyModifiers::CONTROL,
                    LogMsg::BookmarkListScroll(ScrollAction::Viewport(-50)),
                )
                .on_key(
                    |k| k.code == KeyCode::Char('f') && k.modifiers == KeyModifiers::CONTROL,
                    LogMsg::BookmarkListScroll(ScrollAction::Viewport(100)),
                )
                .on_key(
                    |k| k.code == KeyCode::Char('b') && k.modifiers == KeyModifiers::CONTROL,
                    LogMsg::BookmarkListScroll(ScrollAction::Viewport(-100)),
                )
                .on_key(|k| k.code == KeyCode::Enter, LogMsg::BookmarkListSelect)
                .on_key(|k| k.code == KeyCode::Esc, LogMsg::BookmarkListClose)
                .on_key(|k| k.code == KeyCode::Char('?'), LogMsg::Help)
                .into()
        }
    };
    let block = block(inner).bordered().border_type(BorderType::Rounded);

    Element::from(block).map(Message::Log)
}
