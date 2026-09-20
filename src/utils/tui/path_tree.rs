use std::{collections::BTreeMap, path::Path};

use bytestring::ByteString;
use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};
use tui_tree_widget::TreeItem;

/// The changed files of a commit as a component tree, ready for `tui-tree-widget`.
///
/// The raw input is one `<status> <path>` line per file, as emitted by
/// [`JJHandle::diff_files`](crate::utils::jj::JJHandle::diff_files). Paths are split into
/// components and every chain of single child *directories* is folded into its parent label (path
/// compression), so `a/b/c/d/x` and `a/b/c/d/y` render as a single `a/b/c/d/` node, while a
/// directory holding a single file keeps both rows.
///
/// The tree item identifier is always the full path, while the shown label is the compressed name,
/// so the selection survives a label changing shape.
#[derive(Debug, Default)]
pub struct PathTree {
    path2status_file: BTreeMap<ByteString, ByteString>,
    items: Vec<TreeItem<'static, ByteString>>,
}

impl PathTree {
    pub fn new(raw: &[u8]) -> Self {
        let mut root = PathNode::default();
        let mut path2status_file = BTreeMap::new();

        for line in String::from_utf8_lossy(raw).lines() {
            let mut chars = line.chars();
            let Some(status) = chars.next() else {
                continue;
            };
            // only the first space separates, so a path may contain spaces
            let Some(path) = chars.as_str().strip_prefix(' ') else {
                continue;
            };
            if !status.is_ascii_alphabetic() || path.is_empty() {
                continue;
            }

            let path = Path::new(path);
            let line: ByteString = line.into();

            path2status_file.insert(path.to_string_lossy().into_owned().into(), line.clone());
            root.insert(path, line);
        }

        let items = root
            .children
            .into_iter()
            .map(|(component, node)| CompressedNode::compress(component, "", node).into_item())
            .collect();

        Self {
            path2status_file,
            items,
        }
    }

    pub fn get(&self) -> &[TreeItem<'static, ByteString>] {
        &self.items
    }

    pub fn status_file(&self, path: &ByteString) -> Option<ByteString> {
        self.path2status_file.get(path).cloned()
    }
}

/// A path component tree before chains of single child directories are folded into their parent.
#[derive(Debug, Default)]
struct PathNode {
    /// the `<status> <path>` line when the input contains this exact path, `None` for a directory
    /// only needed as a parent
    file: Option<ByteString>,
    children: BTreeMap<String, PathNode>,
}

impl PathNode {
    fn insert(&mut self, path: &Path, file: ByteString) {
        let mut node = self;

        for component in path.components() {
            let component = component.as_os_str().to_string_lossy().into_owned();
            node = node.children.entry(component).or_default();
        }

        node.file = Some(file);
    }

    /// Whether this node is a directory whose unique child is a directory too, i.e. whether the
    /// child folds into this node's label. A single file always keeps its own row.
    fn folds_into_parent(&self) -> bool {
        self.file.is_none()
            && self.children.len() == 1
            && self
                .children
                .values()
                .next()
                .is_some_and(|child| child.file.is_none())
    }
}

/// A [`PathNode`] with every chain of single child *directories* eaten into `label`.
#[derive(Debug)]
struct CompressedNode {
    /// shown name, e.g. `ui/view/log` for a folded directory chain or `files.rs` for a file.
    /// A directory label can contain `/`, a file label never does.
    label: String,
    /// full path, used as the tree item identifier
    path: String,
    /// the `<status> <path>` line of this file, `None` for a directory
    file: Option<ByteString>,
    children: Vec<CompressedNode>,
}

impl CompressedNode {
    fn compress(component: String, parent: &str, mut node: PathNode) -> Self {
        let mut path = join(parent, &component);
        let mut label = component;

        // Fold a chain of directories, but never swallow the file at its end: a directory holding
        // a single file stays a directory row with the file below it, so `templates/keymap.toml`
        // renders as `templates/` + `keymap.toml` instead of one `templates/keymap.toml` row.
        while node.folds_into_parent() {
            let (component, child) = node.children.into_iter().next().expect("len == 1");
            path = join(&path, &component);
            label = join(&label, &component);
            node = child;
        }

        let children = node
            .children
            .into_iter()
            .map(|(component, child)| Self::compress(component, &path, child))
            .collect();

        Self {
            label,
            path,
            file: node.file,
            children,
        }
    }

    fn into_item(self) -> TreeItem<'static, ByteString> {
        let children = self.children.into_iter().map(Self::into_item).collect();
        let identifier = ByteString::from(self.path);
        let text = match self.file {
            // the line is `<status> <path>`, the file row shows the status and the file name
            Some(file) => Line::from(vec![
                Span::styled(
                    format!("{} ", file.chars().next().unwrap_or_default()),
                    status_style(&file),
                ),
                Span::raw(self.label),
            ]),
            None => Line::from(format!("{}/", self.label)),
        };

        TreeItem::new(identifier, text, children).expect("path identifiers must be unique")
    }
}

/// Colors mirroring jj's `diff summary` theme (`colors."diff added"` and friends). The status
/// character is the first one of the `<status> <path>` line.
fn status_style(status_file: &str) -> Style {
    let color = match status_file.as_bytes().first() {
        Some(b'A' | b'C') => Color::Indexed(2), // green
        Some(b'D') => Color::Indexed(1),        // red
        Some(b'M' | b'R') => Color::Indexed(6), // cyan
        _ => return Style::new(),
    };

    Style::new().fg(color)
}

fn join(parent: &str, component: &str) -> String {
    if parent.is_empty() {
        component.into()
    } else {
        format!("{parent}/{component}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compress(files: &[&str]) -> Vec<CompressedNode> {
        let mut root = PathNode::default();

        for line in files {
            let (_status, path) = line.split_once(' ').expect("must be `<status> <path>`");
            root.insert(Path::new(path), (*line).into());
        }

        root.children
            .into_iter()
            .map(|(component, node)| CompressedNode::compress(component, "", node))
            .collect()
    }

    fn dump(nodes: &[CompressedNode]) -> Vec<String> {
        nodes
            .iter()
            .map(|node| {
                let kind = node.file.as_ref().map_or_else(
                    || "dir".into(),
                    |file| file.chars().next().unwrap_or_default().to_string(),
                );
                let mut out = format!("{kind} {} (label {})", node.path, node.label);

                if !node.children.is_empty() {
                    out.push_str(&format!(" [{}]", dump(&node.children).join(", ")));
                }

                out
            })
            .collect()
    }

    #[test]
    fn unary_chains_are_compressed() {
        let tree = compress(&["A a/b/c/d/x", "M a/b/c/d/y"]);

        assert_eq!(
            dump(&tree),
            ["dir a/b/c/d (label a/b/c/d) \
             [A a/b/c/d/x (label x), M a/b/c/d/y (label y)]"]
        );
    }

    #[test]
    fn a_directory_holding_a_single_file_keeps_both_rows() {
        let tree = compress(&["M templates/keymap.toml"]);

        assert_eq!(
            dump(&tree),
            ["dir templates (label templates) \
             [M templates/keymap.toml (label keymap.toml)]"]
        );
    }

    #[test]
    fn sibling_directories_are_kept_apart() {
        let tree = compress(&["A a/b/x", "A a/b/y", "A a/c/z"]);

        assert_eq!(
            dump(&tree),
            ["dir a (label a) \
             [dir a/b (label b) [A a/b/x (label x), A a/b/y (label y)], \
             dir a/c (label c) [A a/c/z (label z)]]"]
        );
    }

    #[test]
    fn a_single_file_folds_only_its_directory_chain() {
        let tree = compress(&["A a/b/c/d/x"]);

        assert_eq!(
            dump(&tree),
            ["dir a/b/c/d (label a/b/c/d) [A a/b/c/d/x (label x)]"]
        );
    }

    #[test]
    fn items_use_the_full_path_as_identifier() {
        let tree = PathTree::new(b"M a/b/c/d/x\nA a/b/c/d/y");
        let items = tree.get();

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].identifier(), &ByteString::from("a/b/c/d"));
        assert_eq!(
            items[0]
                .children()
                .iter()
                .map(|item| item.identifier().to_string())
                .collect::<Vec<_>>(),
            ["a/b/c/d/x", "a/b/c/d/y"]
        );
    }

    #[test]
    fn the_status_of_a_changed_file_is_known() {
        let tree = PathTree::new(b"M a/b/foo bar.c\nD old/foo.c\nA new/foo.c\n");

        assert_eq!(
            tree.status_file(&ByteString::from("a/b/foo bar.c")),
            Some("M a/b/foo bar.c".into())
        );
        assert_eq!(
            tree.status_file(&ByteString::from("old/foo.c")),
            Some("D old/foo.c".into())
        );
        // a directory node is not a changed file
        assert_eq!(tree.status_file(&ByteString::from("old")), None);
        // a renamed file expands into its removed and added paths
        assert_eq!(
            tree.status_file(&ByteString::from("new/foo.c")),
            Some("A new/foo.c".into())
        );
    }

    #[test]
    fn empty_and_broken_lines_are_skipped() {
        let tree = PathTree::new(b" A\nM\n\x00 b.c\nQ \n\nA a/b.c\n");

        assert_eq!(tree.get().len(), 1);
        assert_eq!(
            tree.status_file(&ByteString::from("a/b.c")),
            Some("A a/b.c".into())
        );
    }
}
