use std::ops::Range;

use ratatui::text::Text;
use smol_str::SmolStr;

use crate::utils::tui::BoxText;

#[derive(Debug, Default)]
pub struct LogText {
    text: BoxText,
    changes: Vec<JJChange>,
}

#[derive(Debug, Default)]
pub struct JJChange {
    beacon: char,
    pub range_line: Range<usize>,
    pub id: SmolStr,
}

impl LogText {
    pub fn new(log: Vec<u8>) -> Self {
        let mut text: BoxText = log.into();
        Self::align(&mut text);
        let changes = Self::parse_changes(&text);

        Self { text, changes }
    }

    pub fn text(&self) -> &Text<'_> {
        &self.text
    }

    pub fn beacons(&self) -> &[JJChange] {
        &self.changes
    }

    pub fn find_working(&self) -> Option<(usize, &JJChange)> {
        self.beacons()
            .iter()
            .enumerate()
            .find_map(|(i, v)| v.is_working().then_some((i, v)))
    }

    pub fn find_by_id(&self, id: &str) -> Option<(usize, &JJChange)> {
        let i = self.changes.iter().position(|v| v.id == id)?;
        Some((i, &self.changes[i]))
    }

    pub fn find_by_long_id(&self, id: &str) -> Option<(usize, &JJChange)> {
        let i = match id.split_once('/') {
            Some((id, divergent)) => self.changes.iter().position(|v: &JJChange| {
                v.id.split_once('/')
                    .is_some_and(|(short_id, change_divergent)| {
                        id.starts_with(short_id) && change_divergent == divergent
                    })
            }),
            None => self
                .changes
                .iter()
                .position(|v: &JJChange| id.starts_with(v.id.as_str())),
        }?;
        Some((i, &self.changes[i]))
    }

    pub fn find_by_line(&self, index: usize) -> Option<(usize, &JJChange)> {
        self.changes
            .binary_search_by(|v| {
                index
                    .clamp(v.range_line.start, v.range_line.end)
                    .cmp(&index)
            })
            .ok()
            .map(|i| (i, &self.changes[i]))
            .or_else(|| self.changes.last().map(|v| (self.changes.len() - 1, v)))
    }
}

impl LogText {
    fn parse_changes(log: &Text<'_>) -> Vec<JJChange> {
        let mut changes = vec![];

        let mut skip_line = false;

        for (i_line, line) in log.iter().enumerate() {
            if skip_line {
                skip_line = false;
                continue;
            }

            let mut span_iter = line.iter().peekable();

            let mut change = JJChange::default();
            let mut id_parts = ["", "", ""];

            while let Some(span) = span_iter.next() {
                match change.beacon {
                    '\x00' => {
                        match span
                            .content
                            .chars()
                            .find(|v| matches!(v, '@' | '◆' | '○' | '×' | '~'))
                        {
                            Some('~') => {
                                break;
                            }
                            Some(c) => {
                                change.beacon = c;
                                change.range_line.start = i_line;

                                skip_line = true;
                            }
                            _ => (),
                        }
                    }
                    _ => {
                        if !span_is_empty(&span.content) {
                            id_parts[0] = &span.content;

                            match span_iter.peek() {
                                Some(span) if span_is_empty(&span.content) => {
                                    change.id = id_parts[0].into();
                                }
                                Some(_) => {
                                    let part1 = span_iter.next().expect("must be `Some`");
                                    id_parts[1] = &part1.content;

                                    if let Some(part2) = span_iter.peek()
                                        && part2.content.starts_with('/')
                                    {
                                        id_parts[2] = &part2.content;
                                    }

                                    change.id = id_parts.into_iter().collect();
                                }
                                None => {
                                    change.id = id_parts[0].into();
                                }
                            }

                            if id_is_root(&change.id) {
                                change.range_line.end = i_line + 1;
                            } else {
                                change.range_line.end = i_line + 2;
                            }

                            changes.push(change);
                            break;
                        }
                    }
                }
            }
        }

        changes
    }

    fn align(text: &mut Text<'_>) {
        let twidth = text.width();

        for line in text.iter_mut() {
            let complete_n = twidth - line.width();
            if complete_n > 0 {
                line.push_span(" ".repeat(complete_n));
            }
        }
    }
}

impl JJChange {
    pub fn is_working(&self) -> bool {
        self.beacon == '@'
    }
}

fn span_is_empty(span: &str) -> bool {
    span.as_bytes().iter().all(|&v| v == b' ')
}

fn id_is_root(id: &str) -> bool {
    id.as_bytes().iter().all(|&v| v == b'z')
}
