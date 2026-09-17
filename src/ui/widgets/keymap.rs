use std::{cell::Cell, collections::HashMap, rc::Rc, sync::LazyLock};

use ratatui::{
    crossterm::event::{KeyCode, KeyModifiers},
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::Cell as TableCell,
};
use ratzgo::{
    core::{Area, BindArea, OnKeyBuilder, Widget},
    scroll::ScrollAction,
    widget::{BorderType, Row, ScrollbarParams, TableState, block, scrollbar, table},
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

pub fn keymap<'a>(
    state: &'a mut TableState,
    page: &str,
    area: &'a Rc<Cell<Rect>>,
) -> impl Widget<HelpMsg> + 'a {
    /// Header row plus its `bottom_margin(1)`
    const HEADER_HEIGHT: usize = 2;

    let height = keymap_at(page).len() + HEADER_HEIGHT;
    let offset = state.offset();

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
                TableCell::from(*key).style(
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                TableCell::from(*desc).style(Style::default().fg(Color::Gray)),
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

    block(inner.bind_area(area))
        .bordered()
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

#[derive(Debug, Deserialize)]
pub struct KeyMapItem {
    key: &'static str,
    desc: &'static str,
}
