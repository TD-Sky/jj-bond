use ratatui::text::Text;
use ratzgo::{
    core::*,
    widget::{BorderType, ParagraphState, block, paragraph},
};

use crate::ui::LogMsg;

#[derive(Debug)]
pub struct VState<'a> {
    pub state: &'a mut ParagraphState,
    pub view: Text<'a>,
}

pub fn view<'a>(VState { state, view }: VState<'a>) -> impl Into<Element<'a, LogMsg>> {
    let inner = paragraph(view, state);
    block(inner).bordered().border_type(BorderType::Rounded)
}
