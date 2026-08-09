use ratatui::{
    crossterm::event::{KeyCode, KeyModifiers},
    layout::Constraint,
    style::{Modifier, Style},
    text::Text,
};
use ratzgo::{
    core::*,
    scroll::ScrollAction,
    widget::{BorderType, ListState, MountPoint, block, list},
};

use crate::ui::{Message, TagsMsg, widgets::Modal};

#[derive(Debug)]
pub struct VState<'a> {
    pub state: &'a mut ListState,
    pub view: Text<'a>,
    pub mount_point: &'a MountPoint<Message>,
    pub modal_delete_tag: Option<&'a str>,
}

pub fn view<'a>(
    VState {
        state,
        view,
        modal_delete_tag,
        mount_point,
    }: VState<'a>,
) -> impl Into<Element<'a, TagsMsg>> {
    state.selected_mut().get_or_insert(0);

    if let Some(tag) = modal_delete_tag {
        mount_point.mount(
            Modal::new("Delete Tag", format!("delete tag `{tag}` ?"))
                .on_key(
                    |k| k.code == KeyCode::Char('y'),
                    TagsMsg::DeleteConfirm(true).into(),
                )
                .on_key(
                    |k| k.code == KeyCode::Char('n') || k.code == KeyCode::Esc,
                    TagsMsg::DeleteConfirm(false).into(),
                )
                .on_key(|k| k.code == KeyCode::Char('?'), TagsMsg::Help.into()),
            |area| area.centered(Constraint::Ratio(1, 2), Constraint::Ratio(1, 3)),
        );
    }

    let inner = list(state)
        .items(view)
        .active(true)
        .decorate(|v| v.highlight_style(Style::new().add_modifier(Modifier::REVERSED)))
        .on_key(
            |k| k.code == KeyCode::Char('j'),
            TagsMsg::ScrollList(ScrollAction::Fixed(1)),
        )
        .on_key(
            |k| k.code == KeyCode::Char('k'),
            TagsMsg::ScrollList(ScrollAction::Fixed(-1)),
        )
        .on_key(
            |k| k.code == KeyCode::Char('d') && k.modifiers == KeyModifiers::CONTROL,
            TagsMsg::ScrollList(ScrollAction::Viewport(50)),
        )
        .on_key(|k| k.code == KeyCode::Char('d'), TagsMsg::Delete)
        .on_key(
            |k| k.code == KeyCode::Char('u') && k.modifiers == KeyModifiers::CONTROL,
            TagsMsg::ScrollList(ScrollAction::Viewport(-50)),
        )
        .on_key(
            |k| k.code == KeyCode::Char('f') && k.modifiers == KeyModifiers::CONTROL,
            TagsMsg::ScrollList(ScrollAction::Viewport(100)),
        )
        .on_key(
            |k| k.code == KeyCode::Char('b') && k.modifiers == KeyModifiers::CONTROL,
            TagsMsg::ScrollList(ScrollAction::Viewport(-100)),
        )
        .on_key(|k| k.code == KeyCode::Enter, TagsMsg::ViewHistory)
        .on_key(|k| k.code == KeyCode::Char('?'), TagsMsg::Help);

    block(inner).bordered().border_type(BorderType::Rounded)
}
