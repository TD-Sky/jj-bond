use ratzgo::{
    core::Element,
    event::DefaultContext,
    widget::{BorderType, block, tabs},
};

use crate::ui::{MainState, Message, NavMsg, State};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tab {
    #[default]
    Log,
    Bookmarks,
    Tags,
    Operations,
}

pub fn view(state: Tab) -> impl Into<Element<'static, NavMsg>> {
    let inner =
        tabs!["Log [1]", "Bookmarks [2]", "Tags [3]", "Operations [4]"].select(state as usize);
    block(inner).bordered().border_type(BorderType::Rounded)
}

pub fn update(state: &mut MainState, msg: NavMsg, ctx: &mut DefaultContext<Message, State>) {
    match msg {
        NavMsg::TabSelect(tab) => state.nav_tab = tab,
        NavMsg::TabPrev => tab_prev(&mut state.nav_tab),
        NavMsg::TabNext => tab_next(&mut state.nav_tab),
    }
    ctx.queue().push(Message::Refresh);
}

const TAB_COUNT: usize = 4;

fn tab_prev(tab: &mut Tab) {
    let n = (*tab as usize + TAB_COUNT - 1) % TAB_COUNT;
    *tab = index2tab(n);
}

fn tab_next(tab: &mut Tab) {
    let n = (*tab as usize + 1) % TAB_COUNT;
    *tab = index2tab(n);
}

fn index2tab(n: usize) -> Tab {
    match n {
        0 => Tab::Log,
        1 => Tab::Bookmarks,
        2 => Tab::Tags,
        3 => Tab::Operations,
        _ => unreachable!(),
    }
}
