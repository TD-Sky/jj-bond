use std::{cell::Cell, rc::Rc};

use ratatui::{
    crossterm::event::{KeyCode, KeyModifiers},
    layout::{Constraint, Rect},
    style::{Color, Style},
    text::Text,
};
use ratzgo::{
    component::{
        Block, BorderType, Paragraph, ParagraphState, ScrollbarOrientation, ScrollbarParams, block,
        paragraph, scrollbar,
    },
    core::*,
    scroll::ScrollAction,
    text::Line,
};

use crate::ui::{
    LogMsg,
    view::log::{LogFocus, LogLayout},
};

#[derive(Debug)]
pub struct VState<'a> {
    pub state: &'a mut ParagraphState,
    pub area: Option<&'a Rc<Cell<Rect>>>,
    pub log_focus: &'a LogFocus,
    pub view: Text<'a>,
    pub id: Option<&'a str>,
    pub file: Option<&'a str>,
}

pub fn view<'a>(
    VState {
        state,
        area,
        log_focus,
        view,
        id,
        file,
    }: VState<'a>,
) -> impl Component<LogMsg> + 'a {
    let blocking: &dyn Fn(Paragraph<'_, _>) -> Block<'_, _, Paragraph<'_, _>> = match area {
        Some(viewport) => {
            let (h, w) = (view.height(), view.width());
            let (y, x) = state.scroll;
            &move |inner| {
                block(inner.bind_area(viewport))
                    .widget_right_opt(
                        scrollbar(ScrollbarParams {
                            content_length: h,
                            viewport: Area::Ref(viewport.clone()),
                            position: y as usize,
                        }),
                        {
                            let viewport = viewport.clone();
                            move |area| {
                                (h > viewport.get().height as usize)
                                    .then(|| area.centered_vertically(Constraint::Percentage(90)))
                            }
                        },
                    )
                    .widget_bottom_opt(
                        scrollbar(ScrollbarParams {
                            content_length: w,
                            viewport: Area::Ref(viewport.clone()),
                            position: x as usize,
                        })
                        .orientation(ScrollbarOrientation::HorizontalBottom)
                        .decorate(|v| v.thumb_symbol("/")),
                        {
                            let viewport = viewport.clone();
                            move |area| {
                                (w > viewport.get().width as usize)
                                    .then(|| area.centered_horizontally(Constraint::Percentage(50)))
                            }
                        },
                    )
            }
        }
        None => &|inner| block(inner),
    };

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
        .on_key(
            |k| k.code == KeyCode::Char('h'),
            LogMsg::ScrollHDiff(ScrollAction::Viewport(-25)),
        )
        .on_key(
            |k| k.code == KeyCode::Char('l'),
            LogMsg::ScrollHDiff(ScrollAction::Viewport(25)),
        )
        .on_key(|k| k.code == KeyCode::Char('?'), LogMsg::Help);

    let mut v = blocking(inner);
    if let Some(id) = id {
        v = v.title(Line::from(id).style(Style::default().fg(Color::Indexed(13))));
    }
    if let Some(file) = file {
        v = v.title_top(
            Line::from(file)
                .right_aligned()
                .style(Style::default().fg(Color::Indexed(14))),
        );
    }
    v = v.bordered().border_type(BorderType::Rounded);

    v
}
