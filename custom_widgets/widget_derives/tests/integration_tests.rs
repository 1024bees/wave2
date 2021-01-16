use widget_derives::{MenuBarOption,MenuOption};
use wave2_custom_widgets::widget::menu_bar::{MenuOption,MenuBarOption};
use strum_macros;

#[derive(Debug,Clone,PartialEq,strum_macros::Display,MenuBarOption)]
pub enum TopMenu {
    Edit(EditMenu),
    View(ViewMenu)
}

#[derive(MenuOption,PartialEq,strum_macros::Display,Debug,Clone)]
pub enum ViewMenu {
    Window1,
    Window2,
}


#[derive(MenuOption,PartialEq,strum_macros::Display,Debug,Clone)]
pub enum EditMenu {
    Copy,
    Delete,
    Paste
}

#[test]
fn test_menu_bar_option() {
    assert_eq!(TopMenu::all().len(),2);
    let mut iter = TopMenu::all().into_iter();

    let edit_menu = iter.next().expect("Should be edit!");
    let edit_messages = [TopMenu::Edit(EditMenu::Copy), TopMenu::Edit(EditMenu::Delete), TopMenu::Edit(EditMenu::Paste)];

    edit_messages.iter().cloned()
        .zip(edit_menu.get_children().iter().map(|x| x.to_message()))
        .for_each(|(generated, ground_truth)| assert_eq!(generated,ground_truth));


    let view_menu =  iter.next().expect("Should be view menu!");
    let view_messages = [TopMenu::View(ViewMenu::Window1), TopMenu::View(ViewMenu::Window2)];

    view_messages.iter().cloned()
        .zip(view_menu.get_children().iter().map(|x| x.to_message()))
        .for_each(|(generated, ground_truth)| assert_eq!(generated,ground_truth));
}

#[test]
fn test_menu_option() {
    //This test is more for the helpers that are derived by MenuOption; technically, MenuOption is
    //derived as part of the MenuBarOption derive
    assert_eq!(ViewMenu::base(), ViewMenu::Window1);
    assert_eq!(EditMenu::base(), EditMenu::Copy);

    assert_eq!(ViewMenu::ALL.len(),2);
    assert_eq!(EditMenu::ALL.len(),3);


}

