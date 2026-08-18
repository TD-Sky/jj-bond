use std::{
    any::type_name_of_val,
    ops::{Deref, DerefMut},
};

use ratatui::{
    crossterm::event::KeyEvent,
    prelude::{Widget as _, *},
};
use ratzgo::core::{Widget, *};

pub struct TextArea<'a, Message> {
    state: &'a mut TextAreaState,
    activity: bool,
    on_key: OnKey<'a, Message>,
    on_paste: Option<OnPaste<Message>>,
}

type OnPaste<Message> = Box<
    dyn for<'s> FnOnce(&'s str, &'s mut ratatui_textarea::TextArea<'static>) -> Message + 'static,
>;

impl<'a, Message> std::fmt::Debug for TextArea<'a, Message>
where
    Message: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TextArea")
            .field("state", &self.state)
            .field("activity", &self.activity)
            .field("on_key", &self.on_key)
            .field(
                "on_paste",
                &format_args!("<closure of `{}`>", type_name_of_val(&self.on_paste)),
            )
            .finish()
    }
}

impl<'a, Message> TextArea<'a, Message> {
    pub fn new(state: &'a mut TextAreaState) -> Self {
        TextArea {
            state,
            activity: false,
            on_key: Default::default(),
            on_paste: None,
        }
    }

    pub fn decorate<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut ratatui_textarea::TextArea<'static>),
    {
        f(&mut self.state.base);
        self
    }

    pub fn on_paste<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&str, &mut ratatui_textarea::TextArea<'static>) -> Message + 'static,
    {
        self.on_paste = Some(Box::new(f));
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

    fn handle_paste(&mut self, content: &str) -> Option<Message> {
        self.on_paste.take().map(|f| f(content, self.state))
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
