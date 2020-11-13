use iced::{
    button, scrollable, text_input, Align, Button, Column, Container, Element, Length, Row,
    Scrollable, Text, TextInput,
};
use log::error;
use std::sync::{Arc, Mutex};
use wave2_custom_widgets::widget::cell;
use wave2_custom_widgets::widget::cell::Cell;

pub trait HasChildren {
    fn has_children(&self) -> bool;
}

#[derive(Clone, Debug)]
pub enum Message<M>
where
    M: 'static + Clone,
{
    Toggle,
    AppMessage(M),
    Blank(M),
    Placeholder,
}

struct TreeNode<'a, T, O>
where
    T: 'a + ToString + Clone + HasChildren,
    O: 'a + ToString + Clone + 'static,
{
    children: Vec<TreeNode<'a, T, O>>,
    ui_state: cell::State<O>,
    expanded_button: button::State,
    expanded: bool,
    payload: &'a T,
    options: &'static [O],
}

impl<'a, T, O> TreeNode<'a, T, O>
where
    T: 'a + ToString + Clone + HasChildren,
    O: 'static + ToString + Clone,
{
    pub fn update<M>(&mut self, message: Message<M>)
    where
        M: 'static + Clone + ToString,
    {
        match message {
            _ => panic!("Unhandled"),
        }
    }

    pub fn view<M>(&mut self) -> Element<Message<M>>
    where
        M: 'static + Clone + ToString + Fn(T) -> M,
    {
        let TreeNode { children, ui_state, expanded_button, expanded, payload, options } = self;

        let expander = Button::new(expanded_button, Text::new(if *expanded { "↓" } else { "←" }))
            .on_press(Message::Toggle);

        let root_cell = Cell::new(ui_state, *payload, *options, |T| Message::Placeholder);

        let top_row = if payload.has_children() {
            Row::new().push(expander).push(root_cell).width(Length::Fill).height(Length::Shrink)
        } else {
            Row::new().push(root_cell).width(Length::Fill).height(Length::Shrink)
        };

        if *expanded {
            let mut elements = vec![top_row.into()];
            elements.extend(children.iter_mut().map(|x| x.view()));
            Column::with_children(elements).into()
        } else {
            top_row.into()
        }
    }
}
