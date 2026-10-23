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
#[derive(Debug, usage_rs::Cli)]
#[usage(bin = "jb", version = VERSION, about)]
pub struct Cli {}
