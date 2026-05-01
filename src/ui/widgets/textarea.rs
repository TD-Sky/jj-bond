use std::ops::{Deref, DerefMut};

use ratatui::{
    crossterm::event::KeyEvent,
    prelude::{Widget as _, *},
};
use ratzgo::core::{Widget, *};

#[derive(Debug)]
pub struct TextArea<'a, Message> {
    state: &'a mut TextAreaState,
    activity: bool,
    on_key: OnKey<'a, Message>,
}

impl<'a, Message> TextArea<'a, Message> {
    pub fn new(state: &'a mut TextAreaState) -> Self {
        TextArea {
            state,
            activity: false,
            on_key: Default::default(),
        }
    }

    pub fn decorate<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut ratatui_textarea::TextArea<'static>),
    {
        f(&mut self.state.base);
        self
    }
}

impl<'a, Message> Widget<Message> for TextArea<'a, Message>
where
    Message: std::fmt::Debug,
{
    fn activity(&self) -> bool {
        self.activity
    }

    fn area(&self) -> Rect {
        self.state.area
    }

    fn set_area(&mut self, area: Rect) {
        self.state.area = area;
    }

    fn handle_key(&mut self, key: &KeyEvent) -> Option<Message> {
        self.on_key.key(key)
    }

    fn adapt(&mut self, buf: &mut Buffer) {
        (&self.state.base).render(self.state.area, buf);
    }
}

impl<'a, Message> Activable for TextArea<'a, Message> {
    fn active_mut(&mut self) -> &mut bool {
        &mut self.activity
    }
}

impl<'a, Message> OnKeyBuilder<'a, Message> for TextArea<'a, Message> {
    fn on_key_mut(&mut self) -> &mut OnKey<'a, Message> {
        &mut self.on_key
    }
}

impl<'a, Message> From<TextArea<'a, Message>> for Element<'a, Message>
where
    Message: std::fmt::Debug + 'a,
{
    fn from(widget: TextArea<'a, Message>) -> Self {
        Self::new(widget)
    }
}

#[derive(Debug, Default)]
pub struct TextAreaState {
    base: ratatui_textarea::TextArea<'static>,
    pub area: Rect,
}

impl Deref for TextAreaState {
    type Target = ratatui_textarea::TextArea<'static>;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl DerefMut for TextAreaState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
