use std::{collections::HashMap, sync::LazyLock};

use ratatui::{
    crossterm::event::{KeyCode, KeyModifiers},
    layout::Constraint,
    style::{Color, Modifier, Style},
    widgets::Cell,
};
use ratzgo::{
    core::{Element, OnKeyBuilder},
    scroll::ScrollAction,
    widget::{BorderType, Row, TableState, block, table},
};
use serde::Deserialize;

use crate::ui::HelpMsg;

static KEYMAP: LazyLock<HashMap<&'static str, Vec<KeyMapItem>>> = LazyLock::new(|| {
    toml::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/templates/keymap.toml"
    )))
    .unwrap_or_default()
});

pub fn keymap_at(page: &str) -> &[KeyMapItem] {
    &KEYMAP[page]
}

pub fn keymap<'a>(state: &'a mut TableState, page: &str) -> impl Into<Element<'a, HelpMsg>> {
    let inner = table(state)
        .header(
            Row::new(vec!["Key", "Description"])
                .style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
                .bottom_margin(1),
        )
        .widths([Constraint::Length(16), Constraint::Min(10)])
        .rows(KEYMAP[page].iter().map(|KeyMapItem { key, desc }| {
            Row::new(vec![
                Cell::from(*key).style(
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Cell::from(*desc).style(Style::default().fg(Color::Gray)),
            ])
        }))
        .decorate(|v| {
            v.row_highlight_style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▶ ")
        })
        .on_key_with(|k| {
            let msg = match k.code {
                KeyCode::Char('j') => HelpMsg::Scroll(ScrollAction::Fixed(1)),
                KeyCode::Char('k') => HelpMsg::Scroll(ScrollAction::Fixed(-1)),
                KeyCode::Char('d') if k.modifiers == KeyModifiers::CONTROL => {
                    HelpMsg::Scroll(ScrollAction::Viewport(50))
                }
                KeyCode::Char('u') if k.modifiers == KeyModifiers::CONTROL => {
                    HelpMsg::Scroll(ScrollAction::Viewport(-50))
                }
                KeyCode::Char('f') if k.modifiers == KeyModifiers::CONTROL => {
                    HelpMsg::Scroll(ScrollAction::Viewport(100))
                }
                KeyCode::Char('b') if k.modifiers == KeyModifiers::CONTROL => {
                    HelpMsg::Scroll(ScrollAction::Viewport(-100))
                }
                KeyCode::Esc => HelpMsg::Close,
                _ => return None,
            };
            Some(msg)
        });

    block(inner).bordered().border_type(BorderType::Rounded)
}

#[derive(Debug, Deserialize)]
pub struct KeyMapItem {
    key: &'static str,
    desc: &'static str,
}
