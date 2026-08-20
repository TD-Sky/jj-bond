use ratatui::crossterm::event::KeyEvent;
use ratzgo::scroll::ScrollAction;
use smol_str::SmolStr;

use crate::{
    ui::view::{log::LogLayout, nav::Tab},
    utils::{
        jj::Split,
        tui::{LogText, TreeText},
    },
};

#[derive(Debug)]
pub enum Message {
    Nav(NavMsg),
    Log(LogMsg),
    Bookmarks(BookmarksMsg),
    Tags(TagsMsg),
    Op(OpMsg),
    Notify(NotifyMsg),
    Help(HelpMsg),
    Refresh,
    Exit,
}

#[derive(Debug)]
pub enum NavMsg {
    TabSelect(Tab),
    TabPrev,
    TabNext,
}

#[derive(Debug)]
pub enum LogMsg {
    UpdateHistory(LogText),
    UpdateShow { text: Vec<u8>, version: u32 },
    UpdateFiles(Vec<u8>),
    UpdateDiff { text: Vec<u8>, version: u32 },
    Layout(LogLayout),
    ScrollHistory(ScrollAction),
    ScrollDiff(ScrollAction),
    ScrollHDiff(ScrollAction),
    ScrollShow(ScrollAction),
    ScrollFiles(ScrollAction),
    New { parent: SmolStr },
    Edit { id: SmolStr },
    Desc { id: SmolStr },
    Abandon { id: SmolStr },
    AbandonConfirm(bool),
    Squash { id: SmolStr },
    SquashConfirm(bool),
    Split(Split),
    SplitConfirm(bool),
    Fetch,
    BookmarkListOpen,
    BookmarkListClose,
    BookmarkListSelect,
    BookmarkListScroll(ScrollAction),
    CreatingBookmark { key: KeyEvent },
    TagListOpen,
    TagListClose,
    TagListSelect,
    TagListScroll(ScrollAction),
    CreatingTag { key: KeyEvent },
    Undo,
    UndoConfirm(bool),
    Redo,
    RedoConfirm(bool),
    Yank { id: SmolStr },
    Unyank,
    RebaseListOpen,
    RebaseListClose,
    RebaseListBack,
    RebaseListSelect,
    RebaseListScroll(ScrollAction),
    Rebase { id: SmolStr },
    RebaseConfirm(bool),
    Duplicate { id: SmolStr },
    DuplicateConfirm(bool),
    ResetMode,
    ScrollUnsync(ScrollAction),
    Push,
    PushConfirm(bool),
    Help,
    Paste,
}

#[derive(Debug)]
pub enum BookmarksMsg {
    UpdateTree(TreeText),
    UpdateHistory { text: LogText, version: u32 },
    ScrollTree(ScrollAction),
    ScrollHistory(ScrollAction),
    ScrollRemotes(ScrollAction),
    BookmarkOpen,
    BookmarkClose,
    ViewHistory,
    Track,
    TrackConfirm(bool),
    Untrack,
    Delete,
    DeleteConfirm(bool),
    Help,
}

#[derive(Debug)]
pub enum TagsMsg {
    UpdateTree(TreeText),
    ScrollTree(ScrollAction),
    ScrollHistory(ScrollAction),
    ScrollRemotes(ScrollAction),
    TagOpen,
    TagClose,
    UpdateHistory { text: LogText, version: u32 },
    ViewHistory,
    Track,
    TrackConfirm(bool),
    Untrack,
    Delete,
    DeleteConfirm(bool),
    Push,
    PushConfirm(bool),
    Help,
}

#[derive(Debug)]
pub enum OpMsg {
    Update(Vec<u8>),
    Scroll(ScrollAction),
    Help,
}

#[derive(Debug)]
pub enum NotifyMsg {
    LogEvent(ratzgo::log::Event),
    Confirm,
}

#[derive(Debug)]
pub enum HelpMsg {
    Page(&'static str),
    Scroll(ScrollAction),
    Close,
}

impl From<NavMsg> for Message {
    fn from(v: NavMsg) -> Self {
        Self::Nav(v)
    }
}

impl From<LogMsg> for Message {
    fn from(v: LogMsg) -> Self {
        Self::Log(v)
    }
}

impl From<BookmarksMsg> for Message {
    fn from(v: BookmarksMsg) -> Self {
        Self::Bookmarks(v)
    }
}

impl From<TagsMsg> for Message {
    fn from(v: TagsMsg) -> Self {
        Self::Tags(v)
    }
}

impl From<OpMsg> for Message {
    fn from(v: OpMsg) -> Self {
        Self::Op(v)
    }
}

impl From<NotifyMsg> for Message {
    fn from(v: NotifyMsg) -> Self {
        Self::Notify(v)
    }
}

impl From<HelpMsg> for Message {
    fn from(v: HelpMsg) -> Self {
        Self::Help(v)
    }
}
