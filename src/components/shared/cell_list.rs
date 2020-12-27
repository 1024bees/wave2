use wave2_custom_widgets::widget::cell;
use wave2_custom_widgets::widget::cell::Cell as VizCell;
use iced::{Column, Element}; 
use log::error;

use wave2_custom_widgets::traits::CellOption;

pub struct ListNode<T,O> {
    ui_state: cell::State<O>,
    node_state: ListNodeState,
    payload: T,
}


#[derive(Copy,Clone,Default)]
/// Carries state specific to list node
pub struct ListNodeState {
    /// offset into the CellList
    pub offset: usize,
    /// bool on if this node has or hasn't been seleted 
    pub selected: bool,
}


impl<T,O> ListNode<T,O> 
where
    T: ToString + Clone ,
    O: CellOption
{
    fn new(payload: T, offset: usize) -> Self {
        ListNode {
            payload,
            ui_state: cell::State::default(),
            node_state : ListNodeState{ offset: offset, ..ListNodeState::default()}
        }
    }
    fn view(&mut self, 
        on_click: impl Fn(ListNodeState) -> Box<dyn Fn(&T) -> O::Message + 'static >,
        on_double_click : impl Fn(ListNodeState) -> Box<dyn Fn(&T) -> O::Message + 'static >,
        ) -> Element<O::Message> {
        let ListNode {
            ui_state,
            payload,
            node_state,
            ..
        } = self;
        let click = on_click(node_state.clone());

        let sig_cell = VizCell::new(ui_state, payload)
            .on_click(click)
            .on_double_click(on_double_click(node_state.clone()))
            .override_selected(node_state.selected.clone());

        sig_cell.into()
    }
}





pub struct CellList<T,O> {
    pub nodes: Vec<ListNode<T,O>>
}

impl<'a,T,O> IntoIterator for &'a CellList<T,O> {
    type Item = &'a T;
    type IntoIter = CLIter<'a,T,O>;

    fn into_iter(self) -> Self::IntoIter {
        CLIter {
            list: self,
            index: 0,
        }
    }
}

pub struct CLIter<'a,T,O> {
    list: &'a CellList<T,O>,
    index: usize,
}

impl<'a,T,O> Iterator for CLIter<'a,T,O> {
    type Item = &'a T;
    fn next(&mut self) -> Option<&'a T> {
        match self.list.nodes.get(self.index) {
            Some(node) => {
                self.index +=1;
                Some(&node.payload)
            }
            None => None
        }
    }
}


impl<T,O> CellList<T,O>
where
    T: ToString + Clone,
    O: CellOption
{
    pub fn new<C>(collection: C) -> Self
    where
        C: IntoIterator<Item = T>,
    {

        let nodes = collection
            .into_iter()
            .enumerate()
            .map(|(idx,payload)| ListNode::new(payload,idx))
            .collect();

        Self {
            nodes
        }
    }


    pub fn push(&mut self, cell_payload : T) {
        self.nodes.push(ListNode::new(cell_payload,self.nodes.len()));
    }

    pub fn remove(&mut self, idx: usize) -> T {
        let rv = self.nodes.remove(idx).payload;
        self.nodes.iter_mut().enumerate().for_each(|(idx,payload)| payload.node_state.offset = idx);
        rv
    }

    pub fn get_slice(&self) -> Vec<&T> {
        self.nodes
            .iter()
            .map(|node| &node.payload)
            .collect()

    }


    pub fn view(&mut self, 
        on_click: impl Fn(ListNodeState) -> Box<dyn Fn(&T) -> O::Message + 'static> + Copy,
        on_double_click : impl Fn(ListNodeState) -> Box<dyn Fn(&T) -> O::Message + 'static> + Copy,
    ) -> Element<O::Message> {
        Column::with_children(
            self
            .nodes
            .iter_mut()
            .map(|x| x.view(on_click,on_double_click))
            .collect())
            .into()
    }

    pub fn toggle_selected(&mut self, offset: usize, selected: bool) {
        if let Some(value) = self.nodes.get_mut(offset) {
            value.node_state.selected = selected;
        } else {
            error!("Trying to toggle out of range cell! Failing")
        }
    }
}

impl<T,O> Default for CellList<T,O> {
    fn default() -> Self {
        Self{
            nodes: vec![]
        }
    }

}

