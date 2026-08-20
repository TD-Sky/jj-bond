use clap::Parser;

const VERSION: &str = constcat::concat!(
    env!("CARGO_PKG_VERSION"),
    " (",
    match option_env!("VERGEN_GIT_SHA") {
        Some(s) => s,
        None => "no-git",
    },
    " ",
    env!("VERGEN_BUILD_DATE"),
    ")",
);

/// jujutsu TUI
#[derive(Debug, Parser)]
#[command(version = VERSION, about)]
pub struct Cli {}
