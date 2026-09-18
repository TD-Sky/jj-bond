use ratatui::{
    crossterm::event::{KeyCode, KeyModifiers},
    layout::Constraint,
    widgets::Padding,
};
use ratzgo::{
    component::{BorderType, ScrollbarParams, block, paragraph, scrollbar},
    core::*,
    event::DefaultContext,
    scroll::ScrollAction,
};

use crate::ui::{HelpMsg, MainState, Message, OpMsg, State};

pub fn view<'a>(state: &'a mut MainState) -> impl Component<OpMsg> + 'a {
    let height = state.op_view.height();
    let position = state.op_state.scroll.0 as usize;

    let inner = paragraph(state.op_view.get(), &mut state.op_state)
        .bind_area(&state.op_area)
        .active(true)
        .on_key_with(|k| {
            let msg = match k.code {
                KeyCode::Char('k') => OpMsg::Scroll(ScrollAction::Fixed(-1)),
                KeyCode::Char('j') => OpMsg::Scroll(ScrollAction::Fixed(1)),
                KeyCode::Char('d') if k.modifiers == KeyModifiers::CONTROL => {
                    OpMsg::Scroll(ScrollAction::Viewport(50))
                }
                KeyCode::Char('u') if k.modifiers == KeyModifiers::CONTROL => {
                    OpMsg::Scroll(ScrollAction::Viewport(-50))
                }
                KeyCode::Char('f') if k.modifiers == KeyModifiers::CONTROL => {
                    OpMsg::Scroll(ScrollAction::Viewport(100))
                }
                KeyCode::Char('b') if k.modifiers == KeyModifiers::CONTROL => {
                    OpMsg::Scroll(ScrollAction::Viewport(-100))
                }
                KeyCode::Char('?') => OpMsg::Help,
                _ => return None,
            };
            Some(msg)
        });

    block(inner)
        .bordered()
        .border_type(BorderType::Rounded)
        .decorate(|v| v.padding(Padding::horizontal(1)))
        .widget_right_opt(
            scrollbar(ScrollbarParams {
                content_length: height,
                viewport: Area::Ref(state.op_area.clone()),
                position,
            }),
            {
                let viewport = state.op_area.clone();
                move |area| {
                    (height > viewport.get().height as usize)
                        .then(|| area.centered_vertically(Constraint::Percentage(90)))
                }
            },
        )
}

pub fn update(state: &mut MainState, msg: OpMsg, ctx: &mut DefaultContext<Message, State>) {
    match msg {
        OpMsg::Update(text) => {
            state.op_view = text.into();
        }
        OpMsg::Scroll(action) => {
            state
                .op_state
                .scroll_vertical(action, state.op_view.height());
        }
        OpMsg::Help => {
            ctx.queue().push(HelpMsg::Page("operations"));
        }
    }
}

pub fn refresh(state: &mut MainState, ctx: &mut DefaultContext<Message, State>) {
    let jj_handle = state.jj_handle.clone();
    ctx.queue()
        .spawn_try(async move { jj_handle.operations().await.map(OpMsg::Update) });
}
