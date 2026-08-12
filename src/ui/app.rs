use ratzgo::event::default_event_loop;

use crate::{
    ui::{State, root::*},
    utils::jj::JJHandle,
};

pub async fn run() {
    let mut state = Box::new(State::default());

    state.main.jj_handle = match JJHandle::current() {
        Ok(v) => v,
        Err(e) => {
            tracing::error!(error = %e, "get current jj workspace failed");
            eprintln!(
                "{e}\n\nCurrent directory is not in jj repository, run `jj git init` if need"
            );
            return;
        }
    };

    if let Err(e) = default_event_loop(state, init, update, view).await {
        eprintln!("failed at running: {e}");
    }
}
