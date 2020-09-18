pub mod cell_list;
mod renderers;
mod styles;


/// We need to create both nested menus and some sort of hierarchy navigator,
/// is trait is shared across them, so its here in the root of this crate
pub trait Nested<T> {
    fn GetChildren(&self) -> Option<&[T]>;
}
