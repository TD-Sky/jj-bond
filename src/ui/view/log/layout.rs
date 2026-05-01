use enumflags2::{BitFlags, bitflags};
use ratatui::{layout::Constraint, macros::constraints};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct LogLayout(BitFlags<LogLayoutFlag>);

#[bitflags]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
enum LogLayoutFlag {
    History = 0b001,
    Files = 0b010,
    Diff = 0b100,
}

impl Default for LogLayout {
    fn default() -> Self {
        Self::HISTORY_FILES
    }
}

impl From<BitFlags<LogLayoutFlag>> for LogLayout {
    fn from(value: BitFlags<LogLayoutFlag>) -> Self {
        Self(value)
    }
}

impl LogLayout {
    pub const HISTORY: Self = Self(BitFlags::<LogLayoutFlag>::from_bits_truncate_c(
        LogLayoutFlag::History as u8,
        BitFlags::CONST_TOKEN,
    ));

    pub const HISTORY_FILES: Self = Self(BitFlags::<LogLayoutFlag>::from_bits_truncate_c(
        LogLayoutFlag::History as u8 | LogLayoutFlag::Files as u8,
        BitFlags::CONST_TOKEN,
    ));

    pub const FILES_DIFF: Self = Self(BitFlags::<LogLayoutFlag>::from_bits_truncate_c(
        LogLayoutFlag::Files as u8 | LogLayoutFlag::Diff as u8,
        BitFlags::CONST_TOKEN,
    ));

    pub const DIFF: Self = Self(BitFlags::<LogLayoutFlag>::from_bits_truncate_c(
        LogLayoutFlag::Diff as u8,
        BitFlags::CONST_TOKEN,
    ));

    pub fn constraints(&self) -> &'static [Constraint] {
        [
            (LogLayoutFlag::History.into(), constraints![*=1].as_slice()),
            (
                LogLayoutFlag::History | LogLayoutFlag::Files,
                &constraints![==2/5, ==3/5],
            ),
            (
                LogLayoutFlag::Files | LogLayoutFlag::Diff,
                &constraints![==2/5, ==3/5],
            ),
        ]
        .iter()
        .find_map(|v| (self.0 == v.0).then_some(v.1))
        .unwrap_or(&[])
    }
}
