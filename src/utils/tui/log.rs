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

        'lines: for (i_line, line) in log.iter().enumerate() {
            if skip_line {
                skip_line = false;
                continue;
            }

            let mut span_iter = line.iter();

            let mut change = JJChange::default();
            let mut id_parts = ["", "", ""];

            for span in span_iter.by_ref() {
                match span
                    .content
                    .chars()
                    .find(|v| matches!(v, '@' | '◆' | '○' | '×' | '~'))
                {
                    Some('~') => {
                        continue 'lines;
                    }
                    Some(c) => {
                        change.beacon = c;
                        change.range_line.start = i_line;

                        skip_line = true;

                        break;
                    }
                    _ => (),
                }
            }

            if change.beacon == '\x00' {
                continue;
            }

            while let Some(span) = span_iter.next() {
                if span.content.starts_with(|c: char| c.is_ascii_alphabetic()) {
                    id_parts[0] = &span.content;

                    let span = span_iter
                        .next()
                        .expect("must follow with second change id part");
                    id_parts[1] = &span.content;

                    if let Some(span) = span_iter.next()
                        && span.content.starts_with('/')
                    {
                        id_parts[2] = &span.content;
                    }

                    break;
                }
            }

            change.id = id_parts.into_iter().collect();

            if id_is_root(&change.id) {
                change.range_line.end = i_line + 1;
            } else {
                change.range_line.end = i_line + 2;
            }

            changes.push(change);
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

fn id_is_root(id: &str) -> bool {
    id.as_bytes().iter().all(|&v| v == b'z')
}
