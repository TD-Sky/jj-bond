use ratzgo::event::default_event_loop;

use crate::ui::{State, root::*};

pub async fn run() {
    let state = Box::new(State::default());
    if let Err(e) = default_event_loop(state, init, update, view).await {
        eprintln!("failed at running: {e}");
    }
}
