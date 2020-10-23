use std::sync::Arc;
use wave2_custom_widgets::widget::{cell};
use wave2_custom_widgets::widget::cell::Cell;
use iced::{button, Button, scrollable, text_input, Align, Column,Row, TextInput, Element, Container, Scrollable,Length, Text};
use log::{error};

#[derive(Clone,Debug)]
pub enum Message<M> 
where 
    M: 'static + Clone,
{
    Toggle,
    AppMessage(M),
    Blank(M),
    Placeholer
}



struct TreeNode<'a, T,O>
where
    T : 'a + ToString + Clone,
    O : 'a + ToString + Clone,

{
    children : Vec<TreeNode<'a, T,O>>,
    ui_state: cell::State<O>,
    expanded_button: button::State,
    expanded : bool,
    payload : &'a T,
    options : &'a [O],
}


impl<'a, T, O> TreeNode<'a, T, O>
where
    T: 'a + ToString + Clone,
    O: 'a + ToString + Clone,
    
{
    pub fn update<M>(&mut self, message: Message<M>) 
    where
        M:  'static + Clone + ToString,
    {
        match message {
            Message::Toggle => {
                self.expanded = !self.expanded;
            }
            _ => { panic!("Unhandled") }
        }
    }

    pub fn view<M>(&mut self) -> Element<Message<M>> 
    where
        M:  'static + Clone + ToString,

    {
        let TreeNode {
            children, 
            ui_state,
            expanded_button,
            expanded,
            payload,
            options
        } = self;


        let expander = Button::new(
            expanded_button,
            Text::new(if *expanded {
                        "↓"
                    } else {
                        "←"
                    }));



        let root_cell = Cell::new(
            ui_state,
            *payload,
            *options,
            |T| Message::Placeholer
            );


        Row::new()
            .push(expander)
            .push(root_cell)
            .into()

    }
}

