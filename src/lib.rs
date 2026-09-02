pub mod cli;
pub mod config;
pub mod ui {
    mod app;
    mod state;
    mod view {
        pub mod help;
        pub mod hint;
        pub mod nav;
        pub mod notification;
        pub mod root;
        pub mod log {
            mod bookmark_list;
            mod diff;
            mod files;
            mod focus;
            mod history;
            mod layout;
            mod log;
            mod show;
            mod tag_list;

            pub use focus::*;
            pub use layout::*;
            pub use log::*;
        }
        pub mod operations {
            mod operations;

            pub use operations::*;
        }
        pub mod bookmarks {
            mod bookmarks;
            mod history;
            mod tree;

            pub use bookmarks::*;
        }
        pub mod tags {
            mod history;
            mod tags;
            mod tree;

            pub use tags::*;
        }
    }
    mod widgets {
        mod keymap;
        mod log_history;
        mod modal;
        mod notification;
        pub mod rebase;
        mod textarea;
        mod tree;

        pub use keymap::*;
        pub use log_history::*;
        pub use modal::*;
        pub use notification::*;
        pub use textarea::*;
        pub use tree::*;
    }
    mod message;

    pub use app::run;
    pub use message::*;
    pub use state::*;
    pub use view::root;
}
pub mod utils {
    mod alloc;
    pub mod jj {
        mod command;
        mod handle;
        mod notify;

        pub use command::*;
        pub use handle::*;
        pub use notify::*;
    }
    pub mod tui {
        mod log;
        mod text;
        mod tree;

        pub use log::*;
        pub use text::*;
        pub use tree::*;
    }
    pub mod log;
}
