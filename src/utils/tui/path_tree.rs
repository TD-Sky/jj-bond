use std::{collections::BTreeMap, path, str::Utf8Error};

use ahash::HashSet;
use bytestring::ByteString;
use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};
use tui_tree_widget::TreeItem;

#[derive(Debug, Default)]
pub struct CompactPathTree {
    base: Vec<TreeItem<'static, ByteString>>,
    set: HashSet<ByteString>,
}

#[derive(Debug)]
struct LoosePathTree<'a> {
    text: ByteString,
    base: BTreeMap<&'a str, Node<'a>>,
}

#[derive(Debug)]
struct Node<'a> {
    path: &'a str,
    status_file: Option<ByteString>,
    level: usize,
}

impl CompactPathTree {
    pub fn new(raw: &[u8]) -> Result<Self, Utf8Error> {
        let mut set = HashSet::default();

        let text = ByteString::try_from(raw)?;
        let mut tree = LoosePathTree::new(text.clone());

        for line in text.lines() {
            let Some((_, path)) = line.split_once(' ') else {
                continue;
            };

            let v = text.slice_ref(line);
            set.insert(v.clone());
            tree.insert(path, v);
        }

        Ok(Self {
            base: tree.into_tui(),
            set,
        })
    }

    pub fn get(&self) -> &[TreeItem<'static, ByteString>] {
        &self.base
    }

    pub fn status_file(&self, status_file: &str) -> Option<&ByteString> {
        self.set.get(status_file)
    }
}

impl<'a> LoosePathTree<'a> {
    pub fn new(text: ByteString) -> Self {
        Self {
            base: Default::default(),
            text,
        }
    }

    pub fn insert(&mut self, path: &'a str, status_file: ByteString) {
        let mut node = "";

        for (index, prefix) in path
            .split_inclusive(path::MAIN_SEPARATOR)
            .map(|cpt| &path[..path.substr_range(cpt).unwrap().end])
            .enumerate()
        {
            node = prefix;
            self.base.entry(prefix).or_insert_with(|| Node {
                path: prefix,
                status_file: None,
                level: index + 1,
            });
        }

        self.base
            .get_mut(node)
            .expect("`node` must be valid key")
            .status_file = Some(status_file);
    }

    pub fn into_tui(self) -> Vec<TreeItem<'static, ByteString>> {
        const FAKE_ROOT: &Node<'static> = &Node {
            path: "",
            status_file: None,
            level: 0,
        };

        self.children_of(FAKE_ROOT)
            .map(|v| self.into_tui_rec(v, ""))
            .collect()
    }
}

impl<'a> LoosePathTree<'a> {
    #[expect(clippy::wrong_self_convention)]
    fn into_tui_rec(&self, node: &Node<'_>, parent: &str) -> TreeItem<'static, ByteString> {
        match &node.status_file {
            Some(status_file) => {
                fn status_style(status: &str) -> Style {
                    let color = match status {
                        "A" | "C" => Color::Indexed(2), // green
                        "D" => Color::Indexed(1),       // red
                        "M" | "R" => Color::Indexed(6), // cyan
                        _ => return Style::new(),
                    };

                    Style::new().fg(color)
                }

                let status = &status_file[0..1];
                let line = Line::from(vec![
                    Span::styled(status.to_owned(), status_style(status)),
                    Span::raw(format!(
                        " {}",
                        node.path
                            .rsplit_once(path::MAIN_SEPARATOR)
                            .map(|v| v.1)
                            .unwrap_or(node.path)
                    )),
                ]);
                TreeItem::new(status_file.clone(), line, vec![]).unwrap()
            }

            None => {
                let mut cursor = node;

                loop {
                    match self.children_of(cursor).count() {
                        0 => unreachable!("path tree leaf should be skipped"),
                        1 => {
                            let child = self
                                .children_of(cursor)
                                .next()
                                .expect("`cursor` must have one child");

                            if self.children_of(child).count() == 0 {
                                break;
                            }

                            cursor = child;
                        }
                        _ => break,
                    }
                }

                let path = cursor.path;

                let children = self
                    .children_of(cursor)
                    .map(|child| self.into_tui_rec(child, path))
                    .collect();

                let ident = self.text.slice_ref(path);
                TreeItem::new(ident, path.trim_start_matches(parent).to_owned(), children).unwrap()
            }
        }
    }

    fn children_of(&'a self, parent: &'a Node<'a>) -> impl Iterator<Item = &'a Node<'a>> {
        self.base.range(parent.path..).filter_map(move |(_, v)| {
            (v.path.starts_with(parent.path) && v.level == parent.level + 1).then_some(v)
        })
    }
}
