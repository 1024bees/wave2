use wave2_custom_widgets::widget::cell;
use wave2_custom_widgets::widget::cell::Cell as VizCell;
use iced::{Column, Element}; 
use log::error;

pub struct ListNode<T,O> {
    ui_state: cell::State<O>,
    payload: T,
    pub offset: usize,
    selected: bool,
}

impl<T,O> ListNode<T,O> 
where
    T: ToString + Clone,
    O: ToString + Clone + 'static,
{
    fn new(payload: T, offset: usize) -> Self {
        ListNode {
            payload,
            offset,
            ui_state: cell::State::default(),
            selected: false,
        }
    }
    fn view<Message: 'static>(&mut self, 
        options : &'static [O],
        on_click: impl Fn(&T) -> Message + 'static ,
        on_double_click : impl Fn(&T) -> Message + 'static ,
        ) -> Element<Message> {
        let ListNode {
            ui_state,
            payload,
            selected,
            ..
        } = self;

        let sig_cell = VizCell::new(ui_state, payload, options)
            .on_click(on_click)
            .on_double_click(on_double_click)
            .override_selected(selected.clone());


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


    pub fn view<Message: 'static>(&mut self, 
        options : &'static [O],
        on_click: impl Fn(&T) -> Message + 'static + Copy,
        on_double_click : impl Fn(&T) -> Message + 'static + Copy,
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
            value.selected = selected;
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

