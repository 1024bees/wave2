use wave2_custom_widgets::widget::cell;
use wave2_custom_widgets::widget::cell::Cell as VizCell;
use iced::{Column, Element}; 
use log::error;

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
    O: ToString + Clone + 'static,
{
    fn new(payload: T, offset: usize) -> Self {
        ListNode {
            payload,
            ui_state: cell::State::default(),
            node_state : ListNodeState{ offset: offset, ..ListNodeState::default()}
        }
    }
    fn view<Message: 'static>(&mut self, 
        options : &'static [O],
        on_click: impl Fn(ListNodeState) -> Box<dyn Fn(&T) -> Message + 'static >,
        on_double_click : impl Fn(ListNodeState) -> Box<dyn Fn(&T) -> Message + 'static >,
        ) -> Element<Message> {
        let ListNode {
            ui_state,
            payload,
            node_state,
            ..
        } = self;
        
        
        let click = on_click(node_state.clone());

        let sig_cell = VizCell::new(ui_state, payload, options)
            .on_click(click)
            .on_double_click(on_double_click(node_state.clone()))
            .override_selected(node_state.selected.clone());


        sig_cell.into()
    }
}





pub struct CellList<T,O> {
    pub nodes: Vec<ListNode<T,O>>
}


impl<T,O> CellList<T,O>
where
    T: ToString + Clone,
    O: ToString + Clone + 'static,

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

    pub fn get_slice(&self) -> Vec<&T> {
        self.nodes
            .iter()
            .map(|node| &node.payload)
            .collect()

    }


    pub fn view<Message: 'static>(&mut self, 
        options : &'static [O],
        on_click: impl Fn(ListNodeState) -> Box<dyn Fn(&T) -> Message + 'static> + Copy,
        on_double_click : impl Fn(ListNodeState) -> Box<dyn Fn(&T) -> Message + 'static> + Copy,
    ) -> Element<Message> {
        Column::with_children(
            self
            .nodes
            .iter_mut()
            .map(|x| x.view(options,on_click,on_double_click))
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

