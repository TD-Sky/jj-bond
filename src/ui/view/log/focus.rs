use crate::ui::view::log::LogLayout;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LogFocus {
    #[default]
    History,
    Files,
    Diff,
}

impl LogFocus {
    pub fn is_history(self) -> bool {
        self == Self::History
    }

    pub fn is_files(self) -> bool {
        self == Self::Files
    }

    pub fn is_diff(self) -> bool {
        self == Self::Diff
    }
}

impl From<LogLayout> for LogFocus {
    fn from(layout: LogLayout) -> Self {
        [
            (LogLayout::HISTORY, Self::History),
            (LogLayout::HISTORY_FILES, Self::History),
            (LogLayout::FILES_DIFF, Self::Files),
            (LogLayout::DIFF, Self::Diff),
        ]
        .iter()
        .find_map(|(ly, v)| (*ly == layout).then_some(*v))
        .expect("unreachable")
    }
}
