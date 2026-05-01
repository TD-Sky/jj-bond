use jj_bond::{ui::run, utils::log::init_log};

fn main() {
    init_log();

    compio::runtime::Runtime::new().unwrap().block_on(run());
}
