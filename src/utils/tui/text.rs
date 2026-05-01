use std::{
    mem,
    ops::{Deref, DerefMut},
};

use ansi_to_tui::IntoText;
use ratatui::text::Text;

#[derive(Debug)]
pub struct BoxText {
    _raw: Box<[u8]>,
    base: Text<'static>,
}

impl Deref for BoxText {
    type Target = Text<'static>;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl DerefMut for BoxText {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl Default for BoxText {
    fn default() -> Self {
        Self::from(vec![])
    }
}

impl From<Vec<u8>> for BoxText {
    fn from(raw: Vec<u8>) -> Self {
        let raw: Box<[u8]> = raw.into();
        let base = match raw.as_ref() {
            [] => Text::default(),
            bytes => unsafe {
                mem::transmute::<Text<'_>, Text<'static>>(bytes.to_text().unwrap_or_default())
            },
        };

        Self { _raw: raw, base }
    }
}

impl From<String> for BoxText {
    fn from(raw: String) -> Self {
        Self::from(raw.into_bytes())
    }
}

impl BoxText {
    pub fn get(&self) -> Text<'_> {
        self.base.clone()
    }
}
