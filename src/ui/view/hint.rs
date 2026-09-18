use ratzgo::{
    component::{BorderType, Tabs, block},
    core::Component,
    text::Line,
};
use thin_cell::unsync::ThinCell;

use crate::ui::Message;

#[derive(Debug)]
pub struct State {
    pub fetching: ThinCell<bool>,
    pub pushing: ThinCell<bool>,
}

pub fn view(State { fetching, pushing }: State) -> impl Component<Message> + 'static {
    let mut items = vec![Line::from("Help [?]"), Line::from("Quit [q]")];

    if *fetching.borrow() {
        items.push(Line::from("fetching......"));
    }

    if *pushing.borrow() {
        items.push(Line::from("pushing......"));
    }

    let inner = Tabs::new(items).decorate(|v| v.select(None));

    block(inner).bordered().border_type(BorderType::Rounded)
}
