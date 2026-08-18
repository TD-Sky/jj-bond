use ratatui::{
    crossterm::event::{KeyCode, KeyModifiers},
    macros::constraints,
    style::{Color, Modifier, Style},
    text::{Line, Text},
};
use ratzgo::{
    core::*,
    scroll::ScrollAction,
    widget::{BorderType, Borders, ListState, block, column, line, list, row},
};

use crate::ui::LogMsg;

#[derive(Debug)]
pub struct VState<'a> {
    pub view: &'a Text<'a>,
    pub from_state: &'a mut ListState,
    pub to_state: &'a mut ListState,
    pub from: Option<&'a str>,
}

pub fn view<'a>(
    VState {
        view,
        from_state,
        to_state,
        from,
    }: VState<'a>,
) -> impl Into<Element<'a, LogMsg>> {
    let mut from_view = view.clone();
    if let Some(selected) = from {
        for line in &mut from_view.lines {
            if line.spans[0].content != selected {
                for span in &mut line.spans {
                    span.style = span.style.fg(Color::DarkGray);
                }
            }
        }
    }

    let mut to_view = view.clone();
    if let Some(selected) = from {
        to_view
            .lines
            .retain(|line| line.spans[0].content != selected);
    }

    let from_list = list(from_state)
        .items(from_view)
        .decorate(|v| v.highlight_style(Style::default().add_modifier(Modifier::REVERSED)))
        .active(from.is_none());
    let to_list = list(to_state)
        .items(to_view)
        .decorate(|v| v.highlight_style(Style::default().add_modifier(Modifier::REVERSED)))
        .active(from.is_some());

    let choosing_to = from.is_some();
    let from_title = line(
        Line::from("FROM")
            .style(
                Style::default()
                    .fg(if choosing_to {
                        Color::DarkGray
                    } else {
                        Color::Cyan
                    })
                    .add_modifier(Modifier::BOLD),
            )
            .centered(),
    );
    let from_list = column! [
        constraints![==1, ==100%];
        [
            from_title,
            from_list
        ]
    ];

    let from_list = block(from_list)
        .decorate(|v| v.borders(Borders::RIGHT))
        .border_type(BorderType::Rounded);

    let to_title = line(
        Line::from("TO")
            .style(
                Style::default()
                    .fg(if choosing_to {
                        Color::Cyan
                    } else {
                        Color::DarkGray
                    })
                    .add_modifier(Modifier::BOLD),
            )
            .centered(),
    );
    let to_list = column! [
        constraints![==1, ==100%];
        [
            to_title,
            to_list,
        ]
    ];

    let inner: Element<LogMsg> = row! [
        constraints![*=1, *=1];
        [from_list, to_list]
    ]
    .on_key_with(move |key| {
        let msg = match key.code {
            KeyCode::Char('k') => LogMsg::RebaseListScroll(ScrollAction::Fixed(-1)),
            KeyCode::Char('j') => LogMsg::RebaseListScroll(ScrollAction::Fixed(1)),
            KeyCode::Char('d') if key.modifiers == KeyModifiers::CONTROL => {
                LogMsg::RebaseListScroll(ScrollAction::Viewport(50))
            }
            KeyCode::Char('u') if key.modifiers == KeyModifiers::CONTROL => {
                LogMsg::RebaseListScroll(ScrollAction::Viewport(-50))
            }
            KeyCode::Char('f') if key.modifiers == KeyModifiers::CONTROL => {
                LogMsg::RebaseListScroll(ScrollAction::Viewport(100))
            }
            KeyCode::Char('b') if key.modifiers == KeyModifiers::CONTROL => {
                LogMsg::RebaseListScroll(ScrollAction::Viewport(-100))
            }
            KeyCode::Enter => LogMsg::RebaseListSelect,
            KeyCode::Esc if choosing_to => LogMsg::RebaseListBack,
            KeyCode::Esc => LogMsg::RebaseListClose,
            KeyCode::Char('?') => LogMsg::Help,
            _ => return None,
        };
        Some(msg)
    })
    .into();

    block(inner)
        .bordered()
        .border_type(BorderType::Rounded)
        .title_top(Line::from(" Rebase bookmarks ").centered())
}
