use std::mem;

use bytestring::ByteString;
use tui_tree_widget::TreeItem;

#[derive(Debug, Default)]
pub struct BookmarkTree {
    _raw: ByteString,
    items: Vec<TreeItem<'static, ByteString>>,
}

impl BookmarkTree {
    pub fn new(raw: String) -> Self {
        let raw: ByteString = raw.into();

        let mut items = vec![];

        for line in raw.lines() {
            let span = line.trim_end();
            match span.strip_prefix("  ") {
                None => {
                    let bookmark = TreeItem::new(raw.slice_ref(span), line.trim_end(), vec![])
                        .expect("must be valid");
                    items.push(bookmark);
                }
                Some(remote) => {
                    let remote = TreeItem::new_leaf(raw.slice_ref(remote), remote);
                    items
                        .last_mut()
                        .expect("must be bookmark")
                        .add_child(remote)
                        .expect("must be valid");
                }
            }
        }

        let items =
            unsafe { mem::transmute::<Vec<TreeItem<'_, _>>, Vec<TreeItem<'static, _>>>(items) };

        Self { _raw: raw, items }
    }

    pub fn get(&self) -> &[TreeItem<'static, ByteString>] {
        &self.items
    }
}
