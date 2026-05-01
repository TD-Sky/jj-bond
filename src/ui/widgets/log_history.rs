use std::cmp::Ordering;

use ratatui::{
    crossterm::event::KeyEvent,
    layout::Rect,
    prelude::Buffer,
    style::{Color, Modifier},
    widgets::{Paragraph, Widget as _},
};
use ratzgo::{
    core::*,
    scroll::{ScrollAction, repos_y_anchored},
    text::Line,
};
use smol_str::SmolStr;

use crate::utils::{
    jj::{ExpandRevsetError, JJHandle},
    tui::{JJChange, LogText},
};

#[derive(Debug)]
pub struct LogHistory<'a, Message> {
    text: &'a LogText,
    state: &'a mut LogHistoryState,
    activity: bool,
    on_key: OnKey<'a, Message>,
}

impl<'a, Message> LogHistory<'a, Message> {
    pub fn new(text: &'a LogText, state: &'a mut LogHistoryState) -> Self {
        Self {
            text,
            state,
            activity: false,
            on_key: Default::default(),
        }
    }
}

impl<'a, Message> Widget<Message> for LogHistory<'a, Message>
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
        if self.text.text().height() == 0 {
            return;
        }

        let mut text = self.text.text().clone();

        if let Some(v) = &mut self.state.yanking {
            match v {
                Yanking::One { id } => {
                    if let Some((_, change)) = self.text.find_by_id(id) {
                        for line in text
                            .lines
                            .get_mut(change.range_line.clone())
                            .into_iter()
                            .flatten()
                        {
                            for span in line.iter_mut() {
                                span.style = span.style.fg(Color::Black).bg(Color::Yellow);
                            }
                        }
                    }
                }
                Yanking::Range { ids, .. } => {
                    for id in ids.lines() {
                        let Some((_, change)) = self.text.find_by_long_id(id) else {
                            continue;
                        };

                        for line in text
                            .lines
                            .get_mut(change.range_line.clone())
                            .into_iter()
                            .flatten()
                        {
                            for span in line.iter_mut() {
                                span.style = span.style.fg(Color::Black).bg(Color::Yellow);
                            }
                        }
                    }
                }
            }
        }

        let Some(change) = self.text.beacons().get(self.state.hover) else {
            return;
        };

        for line in text
            .lines
            .get_mut(change.range_line.clone())
            .into_iter()
            .flatten()
        {
            for span in line.iter_mut() {
                span.style = span.style.add_modifier(Modifier::REVERSED);
            }
        }

        Paragraph::new(text)
            .scroll(self.state.scroll)
            .render(self.area(), buf);
    }
}

#[derive(Debug, Default)]
pub struct LogHistoryState {
    /// `(y, x)` offset for scroll
    scroll: (u16, u16),
    hover: usize,
    yanking: Option<Yanking>,
    area: Rect,
}

#[derive(Debug)]
pub enum Yanking {
    One {
        id: SmolStr,
    },
    Range {
        base: (SmolStr, SmolStr),
        ids: Box<str>,
    },
}

impl LogHistoryState {
    pub fn hovered(&self) -> usize {
        self.hover
    }

    pub fn hover(&mut self, index: usize) {
        self.hover = index;
    }

    pub fn reset(&mut self) {
        self.hover(0);
        self.scroll = (0, 0);
    }

    pub async fn yank(&mut self, log: &LogText, jj: &JJHandle, id: &str) {
        match self.yanking.take() {
            None => {
                self.yanking = Some(Yanking::One { id: id.into() });
            }
            Some(Yanking::One { id: prev_id }) => {
                let Some((prev_i, _)) = log.find_by_id(&prev_id) else {
                    return;
                };
                let Some((i, _)) = log.find_by_id(id) else {
                    return;
                };

                let (start, end) = match prev_i.cmp(&i) {
                    Ordering::Less => (id, prev_id.as_str()),
                    Ordering::Greater => (prev_id.as_str(), id),
                    Ordering::Equal => return,
                };

                let revset = format!("{start}::{end}");

                match jj.expand_revset(&revset).await {
                    Ok(ids) => {
                        self.yanking = Some(Yanking::Range {
                            base: (start.into(), end.into()),
                            ids,
                        })
                    }
                    Err(ExpandRevsetError::Invalid) => {
                        self.yanking = Some(Yanking::One { id: prev_id });
                    }
                    Err(e) => {
                        let mut text = e.into_text();
                        text.lines
                            .insert(0, Line::from(format!("revset: {revset}")));
                        ratzgo::log::error("expanding revset", text);
                    }
                }
            }
            Some(Yanking::Range { .. }) => {
                self.yanking = Some(Yanking::One { id: id.into() });
            }
        }
    }

    pub fn unyank(&mut self) -> Option<Yanking> {
        self.yanking.take()
    }

    pub fn yanking(&self) -> Option<&Yanking> {
        self.yanking.as_ref()
    }

    pub fn area(&self) -> Rect {
        self.area
    }

    pub fn scroll_vertical<'a>(
        &mut self,
        log: &'a LogText,
        action: ScrollAction,
    ) -> Option<&'a JJChange> {
        let (i, change) = match action {
            ScrollAction::Fixed(n) => {
                let i = self.hover.checked_add_signed(n as isize)?;
                let change = log.beacons().get(i)?;

                (i, change)
            }
            ScrollAction::Viewport(n) => {
                let beacon = log.beacons().get(self.hover)?;

                let offset = (self.area.height as f32 * n as f32 * 0.01) as isize;
                let line = beacon.range_line.start.saturating_add_signed(offset);
                let (i, change) = log.find_by_line(line)?;

                if change.id == beacon.id {
                    return None;
                }

                (i, change)
            }
        };

        self.hover(i);
        self.scroll.0 = repos_y_anchored(
            self.scroll.0,
            log.text().height(),
            self.area.height,
            4,
            change.range_line.start,
        );

        Some(change)
    }
}

impl<'a, Message> From<LogHistory<'a, Message>> for Element<'a, Message>
where
    Message: std::fmt::Debug + 'a,
{
    fn from(widget: LogHistory<'a, Message>) -> Self {
        Element::new(widget)
    }
}

impl<'a, Message> Activable for LogHistory<'a, Message> {
    fn active_mut(&mut self) -> &mut bool {
        &mut self.activity
    }
}

impl<'a, Message> OnKeyBuilder<'a, Message> for LogHistory<'a, Message> {
    fn on_key_mut(&mut self) -> &mut OnKey<'a, Message> {
        &mut self.on_key
    }
}
