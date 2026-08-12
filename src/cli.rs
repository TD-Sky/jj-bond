use clap::Parser;

const VERSION: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    " (",
    env!("VERGEN_GIT_SHA"),
    " ",
    env!("VERGEN_BUILD_DATE"),
    ")",
);

/// jujutsu TUI
#[derive(Debug, Parser)]
#[command(version = VERSION, about)]
pub struct Cli {}
