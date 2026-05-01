use std::mem;

use ratatui::{
    crossterm::event::KeyEvent,
    macros::constraints,
    prelude::*,
    text::Text,
    widgets::{Block, Padding, Paragraph, Widget as _, Wrap},
};
use ratzgo::core::{Element, OnKey, OnKeyBuilder, Widget};

#[derive(Debug)]
pub struct Notification<'a, Message> {
    title: Line<'a>,
    text: Text<'a>,
    area: Rect,
    on_key: OnKey<'a, Message>,
}

impl<'a, Message> Notification<'a, Message> {
    pub fn new(title: impl Into<Line<'a>>, text: impl Into<Text<'a>>) -> Self {
        Self {
            title: title.into(),
            text: text.into(),
            area: Default::default(),
            on_key: Default::default(),
        }
    }
}

impl<'a, Message> Widget<Message> for Notification<'a, Message>
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
        let [area_content, area_bottom] =
            Layout::vertical(constraints![==100%, ==3]).areas(self.area);

        Paragraph::new(mem::take(&mut self.text))
            .wrap(Wrap { trim: false })
            .block(
                Block::bordered()
                    .title(mem::take(&mut self.title))
                    .padding(Padding::horizontal(1)),
            )
            .render(area_content, buf);
        Paragraph::new("press [Esc] to continue")
            .block(Block::bordered())
            .centered()
            .render(area_bottom, buf);
    }
}

impl<'a, Message> From<Notification<'a, Message>> for Element<'a, Message>
where
    Message: std::fmt::Debug + 'static,
{
    fn from(widget: Notification<'a, Message>) -> Self {
        Element::new(widget)
    }
}

impl<'a, Message> OnKeyBuilder<'a, Message> for Notification<'a, Message> {
    fn on_key_mut(&mut self) -> &mut OnKey<'a, Message> {
        &mut self.on_key
    }
}
