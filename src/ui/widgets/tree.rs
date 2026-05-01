use std::{
    hash::Hash,
    mem,
    ops::{Deref, DerefMut},
};

use ratatui::{
    crossterm::event::KeyEvent,
    prelude::{StatefulWidget as _, *},
};
use ratzgo::{
    core::{Widget, *},
    scroll::ScrollAction,
};
use tui_tree_widget::TreeItem;

#[derive(Debug)]
pub struct Tree<'a, I, Message> {
    base: tui_tree_widget::Tree<'a, I>,
    state: &'a mut TreeState<I>,
    activity: bool,
    on_key: OnKey<'a, Message>,
}

impl<'a, I, Message> Tree<'a, I, Message> {
    pub fn new(items: &'a [TreeItem<'a, I>], state: &'a mut TreeState<I>) -> Self
    where
        I: Clone + PartialEq + Eq + Hash,
    {
        Tree {
            base: tui_tree_widget::Tree::new(items).expect("each item must be unique"),
            state,
            activity: false,
            on_key: Default::default(),
        }
    }

    pub fn decorate<F>(mut self, f: F) -> Self
    where
        F: FnOnce(tui_tree_widget::Tree<'a, I>) -> tui_tree_widget::Tree<'a, I>,
    {
        self.base = f(self.base);
        self
    }
}

impl<'a, I, Message> Widget<Message> for Tree<'a, I, Message>
where
    I: std::fmt::Debug + Clone + PartialEq + Eq + Hash,
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
        let tree = mem::replace(
            &mut self.base,
            tui_tree_widget::Tree::<I>::new(&[]).expect("default tree"),
        );
        tree.render(self.state.area, buf, &mut self.state.base);
    }
}

impl<'a, I, Message> Activable for Tree<'a, I, Message> {
    fn active_mut(&mut self) -> &mut bool {
        &mut self.activity
    }
}

impl<'a, I, Message> OnKeyBuilder<'a, Message> for Tree<'a, I, Message> {
    fn on_key_mut(&mut self) -> &mut OnKey<'a, Message> {
        &mut self.on_key
    }
}

impl<'a, I, Message> From<Tree<'a, I, Message>> for Element<'a, Message>
where
    I: std::fmt::Debug + Clone + PartialEq + Eq + Hash,
    Message: std::fmt::Debug + 'a,
{
    fn from(widget: Tree<'a, I, Message>) -> Self {
        Self::new(widget)
    }
}

#[derive(Debug, Default)]
pub struct TreeState<I> {
    base: tui_tree_widget::TreeState<I>,
    pub area: Rect,
}

impl<I> Deref for TreeState<I> {
    type Target = tui_tree_widget::TreeState<I>;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl<I> DerefMut for TreeState<I> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl<I> TreeState<I>
where
    I: Clone + PartialEq + Eq + Hash,
{
    pub fn scroll_vertical(&mut self, action: ScrollAction) {
        let selected_offset = match action {
            ScrollAction::Fixed(n) => n,
            ScrollAction::Viewport(n) => (self.area.height as f32 * n as f32 * 0.01) as i16,
        };

        self.base.select_relative(|v| match v {
            Some(v) => v.saturating_add_signed(selected_offset as isize),
            None if selected_offset >= 0 => selected_offset as usize,
            _ => 0,
        });

        self.base.scroll_selected_into_view();
    }
}
