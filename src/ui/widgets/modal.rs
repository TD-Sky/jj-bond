use std::mem;

use ratatui::{
    crossterm::event::KeyEvent,
    macros::{constraint, constraints},
    prelude::*,
    text::Text,
    widgets::{Block, BorderType, Padding, Paragraph, Widget as _, Wrap},
};
use ratzgo::{
    core::{Element, OnKey, OnKeyBuilder, Widget},
    widget::Borders,
};

#[derive(Debug)]
pub struct Modal<'a, Message> {
    title: Line<'a>,
    text: Text<'a>,
    area: Rect,
    on_key: OnKey<'a, Message>,
}

impl<'a, Message> Modal<'a, Message> {
    pub fn new(title: impl Into<Line<'a>>, text: impl Into<Text<'a>>) -> Self {
        Self {
            title: title.into().centered(),
            text: text.into(),
            area: Default::default(),
            on_key: Default::default(),
        }
    }
}

impl<'a, Message> Widget<Message> for Modal<'a, Message>
where
    Message: std::fmt::Debug,
{
    fn activity(&self) -> bool {
        true
    }

    fn area(&self) -> Rect {
        self.area
    }

    fn set_area(&mut self, area: Rect) {
        self.area = area;
    }

    fn handle_key(&mut self, key: &KeyEvent) -> Option<Message> {
        self.on_key.key(key)
    }

    fn adapt(&mut self, buf: &mut Buffer) {
        let outer = Block::bordered()
            .border_type(BorderType::Rounded)
            .title(mem::take(&mut self.title).centered());

        let [area_content, area_bottom] =
            Layout::vertical(constraints![==100%, ==1]).areas(outer.inner(self.area));
        let [area_bottom_left, area_bottom_right] =
            Layout::horizontal(constraints![*=1, *=1]).areas(area_bottom);

        Paragraph::new(mem::take(&mut self.text))
            .wrap(Wrap { trim: false })
            .block(
                Block::default()
                    .borders(Borders::BOTTOM)
                    .padding(Padding::horizontal(1)),
            )
            .render(area_content, buf);

        Line::from("[Y]es").centered().render(area_bottom_left, buf);
        Line::from("(N)o").centered().render(area_bottom_right, buf);

        outer.render(self.area, buf);
    }
}

impl<'a, Message> From<Modal<'a, Message>> for Element<'a, Message>
where
    Message: std::fmt::Debug + 'static,
{
    fn from(widget: Modal<'a, Message>) -> Self {
        Element::new(widget)
    }
}

impl<'a, Message> OnKeyBuilder<'a, Message> for Modal<'a, Message> {
    fn on_key_mut(&mut self) -> &mut OnKey<'a, Message> {
        &mut self.on_key
    }
}

pub fn modal_area(area: Rect) -> Rect {
    area.centered(constraint!(==40%), constraint!(==40%))
}
