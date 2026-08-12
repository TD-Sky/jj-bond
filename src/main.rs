use clap::Parser;
use jj_bond::{cli::Cli, ui::run, utils::log::init_log};

fn main() {
    init_log();

    Cli::parse();

    compio::runtime::Runtime::new().unwrap().block_on(run());
}
