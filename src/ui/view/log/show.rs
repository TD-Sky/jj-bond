use std::{cell::Cell, rc::Rc};

use ratatui::{
    layout::{Constraint, Rect},
    text::Text,
};
use ratzgo::{
    component::{BorderType, ParagraphState, ScrollbarParams, block, paragraph, scrollbar},
    core::*,
};

use crate::ui::LogMsg;

#[derive(Debug)]
pub struct VState<'a> {
    pub state: &'a mut ParagraphState,
    pub area: &'a Rc<Cell<Rect>>,
    pub view: Text<'a>,
}

pub fn view<'a>(VState { state, area, view }: VState<'a>) -> impl Component<LogMsg> + 'a {
    let height = view.height();
    let position = state.scroll.0 as usize;

    let inner = paragraph(view, state).bind_area(area);

    block(inner)
        .bordered()
        .border_type(BorderType::Rounded)
        .widget_right_opt(
            scrollbar(ScrollbarParams {
                content_length: height,
                viewport: Area::Ref(area.clone()),
                position,
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
