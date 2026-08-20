use std::cell::OnceCell;

use bytestring::ByteString;
use ratzgo::{
    event::UnsyncDebounce,
    widget::{ListState, MountPoint, ParagraphState},
};
use smol_str::SmolStr;
use thin_cell::unsync::ThinCell;

use crate::{
    config::Config,
    ui::{
        Message,
        view::{
            help,
            log::{LogFocus, LogLayout},
            nav::Tab,
            notification,
        },
        widgets::{LogHistoryState, TextAreaState, TreeState},
    },
    utils::{
        jj::{Abandon, Duplicate, JJHandle, LogMode, Rebase, Split, Squash},
        tui::{BoxText, LogText, TreeText},
    },
};

#[derive(Debug, Default)]
pub struct State {
    pub config: Config,
    pub main: MainState,
    pub notify: notification::State,
    pub help: help::State,
}

#[derive(Debug, Default)]
pub struct MainState {
    pub jj_handle: JJHandle,
    pub nav_tab: Tab,
    pub mount_point: MountPoint<Message>,
    pub log_mode: LogMode,
    pub log_layout: LogLayout,
    pub log_focus: LogFocus,
    pub log_history: LogText,
    pub log_history_state: LogHistoryState,
    pub log_show_debounce: OnceCell<UnsyncDebounce<Message>>,
    pub log_show_state: ParagraphState,
    pub log_show_view: BoxText,
    pub log_files_state: ListState,
    pub log_files_view: BoxText,
    pub log_diff_debounce: OnceCell<UnsyncDebounce<Message>>,
    pub log_diff_state: ParagraphState,
    pub log_diff_view: BoxText,
    pub log_abandon: Option<Abandon>,
    pub log_squash: Option<Squash>,
    pub log_rebase: Option<Rebase>,
    pub log_split: Option<Split>,
    pub log_duplicate: Option<Duplicate>,
    pub log_reloc: LogRelocate,
    pub log_fetching: ThinCell<bool>,
    pub log_modal_rebase_list: Option<BoxText>,
    pub log_modal_rebase_list_state: (ListState, ListState),
    pub log_modal_rebase_from: Option<SmolStr>,
    pub log_modal_bookmark_list: Option<BoxText>,
    pub log_modal_bookmark_list_state: (ListState, Option<TextAreaState>),
    pub log_modal_undo_state: bool,
    pub log_modal_redo_state: bool,
    pub log_modal_tag_list: Option<BoxText>,
    pub log_modal_tag_list_state: (ListState, Option<TextAreaState>),
    pub log_pushing: ThinCell<bool>,
    pub log_modal_unsync: Option<BoxText>,
    pub log_modal_unsync_state: ListState,
    pub bookmarks: TreeText,
    pub bookmarks_state: TreeState<ByteString>,
    pub bookmarks_history_view: LogText,
    pub bookmarks_history_debounce: OnceCell<UnsyncDebounce<Message>>,
    pub bookmarks_history_state: LogHistoryState,
    pub bookmarks_modal_delete: Option<ByteString>,
    pub bookmarks_modal_remotes: Option<BookmarkTrack>,
    pub bookmarks_modal_remotes_state: ListState,
    pub tags: TreeText,
    pub tags_state: TreeState<ByteString>,
    pub tags_history_view: LogText,
    pub tags_history_debounce: OnceCell<UnsyncDebounce<Message>>,
    pub tags_history_state: LogHistoryState,
    pub tags_modal_delete: Option<ByteString>,
    pub tags_modal_push: Option<TagPush>,
    pub tags_modal_remotes: Option<TagTrack>,
    pub tags_modal_remotes_state: ListState,
    pub op_view: BoxText,
    pub op_state: ParagraphState,
}

impl MainState {
    pub fn log_show_debounce(&self) -> &UnsyncDebounce<Message> {
        self.log_show_debounce
            .get()
            .expect("`OnceCell` must be init")
    }

    pub fn log_show_debounce_mut(&mut self) -> &mut UnsyncDebounce<Message> {
        self.log_show_debounce
            .get_mut()
            .expect("`OnceCell` must be init")
    }

    pub fn log_diff_debounce(&self) -> &UnsyncDebounce<Message> {
        self.log_diff_debounce
            .get()
            .expect("`OnceCell` must be init")
    }

    pub fn log_diff_debounce_mut(&mut self) -> &mut UnsyncDebounce<Message> {
        self.log_diff_debounce
            .get_mut()
            .expect("`OnceCell` must be init")
    }

    pub fn bookmarks_history_debounce(&self) -> &UnsyncDebounce<Message> {
        self.bookmarks_history_debounce
            .get()
            .expect("`OnceCell` must be init")
    }

    pub fn bookmarks_history_debounce_mut(&mut self) -> &mut UnsyncDebounce<Message> {
        self.bookmarks_history_debounce
            .get_mut()
            .expect("`OnceCell` must be init")
    }

    pub fn tags_history_debounce(&self) -> &UnsyncDebounce<Message> {
        self.tags_history_debounce
            .get()
            .expect("`OnceCell` must be init")
    }

    pub fn tags_history_debounce_mut(&mut self) -> &mut UnsyncDebounce<Message> {
        self.tags_history_debounce
            .get_mut()
            .expect("`OnceCell` must be init")
    }
}

#[derive(Debug, Default, Clone)]
pub enum LogRelocate {
    Concrete {
        id: SmolStr,
        file: Option<SmolStr>,
    },
    #[default]
    Working,
    Index {
        index: usize,
        file: Option<SmolStr>,
    },
}

impl LogRelocate {
    pub fn file(&self) -> Option<&str> {
        match self {
            LogRelocate::Concrete { file, .. } => file.as_deref(),
            LogRelocate::Working => None,
            LogRelocate::Index { file, .. } => file.as_deref(),
        }
    }
}

#[derive(Debug, Default)]
pub struct BookmarkTrack {
    pub bookmark: ByteString,
    pub remotes: Vec<SmolStr>,
}

#[derive(Debug, Default)]
pub struct TagTrack {
    pub tag: ByteString,
    pub remotes: Vec<SmolStr>,
}

#[derive(Debug, Default)]
pub struct TagPush {
    pub name: ByteString,
    pub remote: ByteString,
}
