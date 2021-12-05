use iced::Text;
use iced::{Column, Element};
use log::error;
use wave2_custom_widgets::core::cell2::DEFAULT_TEXT_SIZE;
use wave2_custom_widgets::widget::cell2;
use wave2_custom_widgets::widget::cell2::Cell2 as VizCell;

use wave2_custom_widgets::traits::CellOption;

pub struct ListNode {
    ui_state: cell2::State,
    node_state: ListNodeState,
}

#[derive(Copy, Clone, Default)]
/// Carries state specific to list node
pub struct ListNodeState {
    /// offset into the CellList
    pub offset: usize,
    /// bool on if this node has or hasn't been seleted
    pub selected: bool,
}

impl ListNode {
    fn new(offset: usize) -> Self {
        ListNode {
            ui_state: cell2::State::default(),
            node_state: ListNodeState {
                offset,
                ..ListNodeState::default()
            },
        }
    }
    fn view<Message: Clone + 'static>(
        &mut self,
        payload: String,
        on_click: impl Fn(ListNodeState) -> Box<dyn Fn() -> Message + 'static>,
        on_double_click: impl Fn(ListNodeState) -> Box<dyn Fn() -> Message + 'static>,
        text_size: Option<u16>,
        cell_padding: Option<u16>,
    ) -> Element<Message> {
        let ListNode {
            ui_state,
            node_state,
            ..
        } = self;
        let click = on_click(node_state.clone());
        let sig_cell = VizCell::new(
            Text::new(payload)
                .size(text_size.unwrap_or(DEFAULT_TEXT_SIZE))
                .width(iced::Length::Fill)
                .into(),
            ui_state,
        )
        .set_width(iced::Length::Fill)
        .set_single_click(click)
        .set_double_click(on_double_click(node_state.clone()))
        .override_selected(node_state.selected.clone());
        //.text_size(text_size)

        sig_cell.into()
    }
}

pub struct CellList {
    pub nodes: Vec<ListNode>,
    text_size: Option<u16>,
    cell_padding: Option<u16>,
    spacing: u16,
}

impl CellList {
    pub fn new(size: usize) -> Self {
        let nodes = (0..size).map(|idx| ListNode::new(idx)).collect();

        Self {
            nodes,
            ..CellList::default()
        }
    }

    pub fn push(&mut self) {
        self.nodes.push(ListNode::new(self.nodes.len()));
    }

    pub fn remove(&mut self, idx: usize) {
        self.nodes.remove(idx);
        self.nodes
            .iter_mut()
            .enumerate()
            .for_each(|(idx, payload)| payload.node_state.offset = idx);
    }

    pub fn view<'a, T: ToString + 'a, Message: Clone + 'static>(
        &mut self,
        strings: impl IntoIterator<Item = &'a T>,
        on_click: impl Fn(ListNodeState) -> Box<dyn Fn() -> Message + 'static> + Copy,
        on_double_click: impl Fn(ListNodeState) -> Box<dyn Fn() -> Message + 'static> + Copy,
    ) -> Element<Message> {
        // To hack around the borrow checker being a little baby. Waa Waa
        let text_size = self.text_size;
        let cell_padding = self.cell_padding;

        let vecs = self
            .nodes
            .iter_mut()
            .zip(strings.into_iter())
            .map(|(x, val)| {
                x.view(
                    (val).to_string(),
                    on_click,
                    on_double_click,
                    text_size,
                    cell_padding,
                )
            })
            .collect();

        Column::with_children(vecs)
            .spacing(self.spacing)
            .padding(10)
            .width(iced::Length::Fill)
            .into()
    }

    pub fn toggle_selected(&mut self, offset: usize, selected: bool) {
        if let Some(value) = self.nodes.get_mut(offset) {
            value.node_state.selected = selected;
        } else {
            error!("Trying to toggle out of range cell! Failing")
        }
    }

    pub fn set_text_size(mut self, text_size: u16) -> Self {
        self.text_size = Some(text_size);
        self
    }

    /// Sets padding per cell (as in between the top of the cell, the text, and the bottom of the
    /// cell
    pub fn set_cell_padding(mut self, padding: u16) -> Self {
        self.cell_padding = Some(padding);
        self
    }

    pub fn column_padding(mut self, padding: u16) -> Self {
        self.spacing = padding;
        self
    }
    pub fn set_spacing(mut self, padding: u16) -> Self {
        self.spacing = padding;
        self
    }
    pub fn clear(&mut self) {
        self.nodes.clear();
    }
}

impl Default for CellList {
    fn default() -> Self {
        Self {
            nodes: vec![],
            cell_padding: None,
            spacing: 0,
            text_size: None,
        }
    }
}
