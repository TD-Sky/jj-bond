use std::{cell::Cell, rc::Rc};

use bytestring::ByteString;
use ratatui::{
    crossterm::event::{KeyCode, KeyModifiers},
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    text::Text,
};
use ratzgo::{
    component::{
        BorderType, ListState, ScrollbarParams, Tabs, block, list as list_widget, scrollbar,
    },
    core::*,
    scroll::ScrollAction,
    text::Line,
};

use crate::{
    ui::{
        FilesView, LogMsg,
        view::log::{LogFocus, LogLayout},
        widgets::{Tree, TreeState},
    },
    utils::tui::PathTree,
};

/// Titles of the tabs on the top border, in the order of `FilesView`'s variants.
const TABS: [&str; 2] = ["list", "tree"];

#[derive(Debug)]
pub struct VState<'a> {
    pub state: &'a mut ListState,
    pub area: &'a Rc<Cell<Rect>>,
    pub log_focus: &'a LogFocus,
    pub view: Text<'a>,
    pub tab: FilesView,
    pub tree_view: &'a PathTree,
    pub tree_state: &'a mut TreeState<ByteString>,
    pub id: Option<&'a str>,
}

pub fn view<'a>(
    VState {
        state,
        area,
        log_focus,
        view,
        tab,
        tree_view,
        tree_state,
        id,
    }: VState<'a>,
) -> impl Component<LogMsg> + 'a {
    let content_height = match tab {
        FilesView::List => view.height(),
        FilesView::Tree => tree_state.flatten(tree_view.get()).len(),
    };
    let position = match tab {
        FilesView::List => state.position.clone(),
        FilesView::Tree => tree_state.position.clone(),
    };

    let inner: Box<dyn Component<LogMsg> + 'a> = match tab {
        FilesView::List => list(state, area, log_focus, view).boxed(),
        FilesView::Tree => tree(tree_state, tree_view, area, log_focus).boxed(),
    };

    let mut v = block(inner);
    if let Some(id) = id {
        v = v.title(Line::from(id).style(Style::default().fg(Color::Indexed(13))));
    }

    v.bordered()
        .border_type(BorderType::Rounded)
        .on_key(
            |k| k.code == KeyCode::Tab,
            LogMsg::FilesViewSelect(toggled(tab)),
        )
        .widget_top(tabs(tab), tabs_area)
        .widget_right_opt(
            scrollbar(ScrollbarParams {
                content_length: content_height,
                viewport: Area::Ref(area.clone()),
                position,
            }),
            {
                let viewport = area.clone();
                move |area| {
                    (content_height > viewport.get().height as usize)
                        .then(|| area.centered_vertically(Constraint::Percentage(90)))
                }
            },
        )
}

fn tabs<'a>(tab: FilesView) -> impl Component<LogMsg> + 'a {
    Tabs::new(TABS.iter().map(|title| Line::from(*title)))
        .select(tab as usize)
        // indexed red keeps the terminal palette and no background color
        .decorate(|v| v.highlight_style(Style::new().fg(Color::Indexed(1))))
}

/// `ratatui` renders tabs left aligned inside the area they are given, so the width has to match
/// the titles exactly for them to sit against the right end of the border.
/// Each title is padded by one space on both sides and separated by a one column divider.
fn tabs_area(border: Rect) -> Rect {
    // stop before the right border corner
    let end = border.right().saturating_sub(1);
    let width = TABS
        .iter()
        .map(|title| Line::from(*title).width() as u16 + 2)
        .sum::<u16>()
        + TABS.len() as u16
        - 1;
    let width = width.min(end.saturating_sub(border.x));

    Rect {
        x: end.saturating_sub(width),
        width,
        ..border
    }
}

fn list<'a>(
    state: &'a mut ListState,
    area: &'a Rc<Cell<Rect>>,
    log_focus: &'a LogFocus,
    view: Text<'a>,
) -> impl Component<LogMsg> + 'a {
    list_widget(state)
        .items(view)
        .bind_area(area)
        .active(log_focus.is_files())
        .decorate(|v| v.highlight_style(Style::new().add_modifier(Modifier::REVERSED)))
        .on_key(
            |k| k.code == KeyCode::Char('j'),
            LogMsg::ScrollFiles(ScrollAction::Fixed(1)),
        )
        .on_key(
            |k| k.code == KeyCode::Char('k'),
            LogMsg::ScrollFiles(ScrollAction::Fixed(-1)),
        )
        .on_key(
            |k| k.code == KeyCode::Char('d') && k.modifiers == KeyModifiers::CONTROL,
            LogMsg::ScrollFiles(ScrollAction::Viewport(50)),
        )
        .on_key(
            |k| k.code == KeyCode::Char('u') && k.modifiers == KeyModifiers::CONTROL,
            LogMsg::ScrollFiles(ScrollAction::Viewport(-50)),
        )
        .on_key(
            |k| k.code == KeyCode::Char('f') && k.modifiers == KeyModifiers::CONTROL,
            LogMsg::ScrollFiles(ScrollAction::Viewport(100)),
        )
        .on_key(
            |k| k.code == KeyCode::Char('b') && k.modifiers == KeyModifiers::CONTROL,
            LogMsg::ScrollFiles(ScrollAction::Viewport(-100)),
        )
        .on_key(
            |k| k.code == KeyCode::Enter,
            LogMsg::Layout(LogLayout::DIFF),
        )
        .on_key(
            |k| k.code == KeyCode::Esc,
            LogMsg::Layout(LogLayout::HISTORY_FILES),
        )
        .on_key(|k| k.code == KeyCode::Char('?'), LogMsg::Help)
}

fn tree<'a>(
    state: &'a mut TreeState<ByteString>,
    view: &'a PathTree,
    area: &'a Rc<Cell<Rect>>,
    log_focus: &'a LogFocus,
) -> impl Component<LogMsg> + 'a {
    Tree::new(view.get(), state)
        .bind_area(area)
        .active(log_focus.is_files())
        .decorate(|v| v.highlight_style(Style::new().add_modifier(Modifier::REVERSED)))
        .on_key(
            |k| k.code == KeyCode::Char('j'),
            LogMsg::ScrollFilesTree(ScrollAction::Fixed(1)),
        )
        .on_key(
            |k| k.code == KeyCode::Char('k'),
            LogMsg::ScrollFilesTree(ScrollAction::Fixed(-1)),
        )
        .on_key(
            |k| k.code == KeyCode::Char('d') && k.modifiers == KeyModifiers::CONTROL,
            LogMsg::ScrollFilesTree(ScrollAction::Viewport(50)),
        )
        .on_key(
            |k| k.code == KeyCode::Char('u') && k.modifiers == KeyModifiers::CONTROL,
            LogMsg::ScrollFilesTree(ScrollAction::Viewport(-50)),
        )
        .on_key(
            |k| k.code == KeyCode::Char('f') && k.modifiers == KeyModifiers::CONTROL,
            LogMsg::ScrollFilesTree(ScrollAction::Viewport(100)),
        )
        .on_key(
            |k| k.code == KeyCode::Char('b') && k.modifiers == KeyModifiers::CONTROL,
            LogMsg::ScrollFilesTree(ScrollAction::Viewport(-100)),
        )
        .on_key(|k| k.code == KeyCode::Char('l'), LogMsg::FilesTreeOpen)
        .on_key(|k| k.code == KeyCode::Char('h'), LogMsg::FilesTreeClose)
        .on_key(
            |k| k.code == KeyCode::Enter,
            LogMsg::Layout(LogLayout::DIFF),
        )
        .on_key(
            |k| k.code == KeyCode::Esc,
            LogMsg::Layout(LogLayout::HISTORY_FILES),
        )
        .on_key(|k| k.code == KeyCode::Char('?'), LogMsg::Help)
}

fn toggled(tab: FilesView) -> FilesView {
    match tab {
        FilesView::List => FilesView::Tree,
        FilesView::Tree => FilesView::List,
    }
}
