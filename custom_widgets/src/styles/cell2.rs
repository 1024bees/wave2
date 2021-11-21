//! Cell2 style
//!
use iced::{Background, Color, Vector};
/// The appearance of a [`Cell2`](crate::widget::cell2::Cell2).
#[derive(Clone, Copy, Debug)]
pub struct Style {
    /// The shadow offset of the [`Cell2`](crate::widget::cell2::Cell2).
    pub shadow_offset: Vector,
    /// The background of the [`Cell2`](crate::widget::cell2::Cell2).
    pub background: Background,
    /// The border radius of the [`Cell2`](crate::widget::cell2::Cell2).
    pub border_radius: f32,
    /// The border width of the [`Cell2`](crate::widget::cell2::Cell2).
    pub border_width: f32,
    /// The border color of the [`Cell2`](crate::widget::cell2::Cell2).
    pub border_color: Color,
    /// The background of the label of the [`Section`](crate::widget::cell2::menu::Section)s.
    pub label_background: Option<Background>,
    /// The text color of the [`Cell2`](crate::widget::cell2::Cell2).
    pub text_color: Color,

    /// The shadow offset of the [`Cell2Overlay`](crate::widget::overlay::cell_overlay::Cell2Overlay).
    pub overlay_shadow_offset: Vector,
    /// The background  of the [`Cell2Overlay`](crate::widget::overlay::cell_overlay::Cell2Overlay).
    pub overlay_background: Background,
    /// The border radius  of the [`Cell2Overlay`](crate::widget::overlay::cell_overlay::Cell2Overlay).
    pub overlay_border_radius: f32,
    /// The border width  of the [`Cell2Overlay`](crate::widget::overlay::cell_overlay::Cell2Overlay).
    pub overlay_border_width: f32,
    /// The border color  of the [`Cell2Overlay`](crate::widget::overlay::cell_overlay::Cell2Overlay).
    pub overlay_border_color: Color,
    /// The background of the label of the [entries](crate::widget::cell2::menu::Entry).
    pub overlay_label_background: Option<Background>,
    /// The text color  of the [`Cell2Overlay`](crate::widget::overlay::cell_overlay::Cell2Overlay).
    pub overlay_text_color: Color,

    /// The corner radius of the separator.
    pub separator_radius: f32,
    /// The width of the separator.
    pub separator_width: f32,
    /// The color of the separator.
    pub separator_color: Color,
    /// The horizontal marging of the separator.
    pub separator_horizontal_margin: f32,
}

/// The appearance of a [`Cell2`](crate::widget::cell2::Cell2).
pub trait StyleSheet {
    /// The normal appearance of a [`Cell2`](crate::widget::cell2::Cell2).
    fn active(&self) -> Style;

    /// The appearance when something is selected of the
    /// [`Cell2`](crate::widget::cell2::Cell2) (Currently unused).
    fn selected(&self) -> Style;

    /// The appearance when something is hovered of the
    /// [`Cell2`](crate::widget::cell2::Cell2).
    fn hovered(&self) -> Style;

    /// The appearance when something is focused of the
    /// [`Cell2`](crate::widget::cell2::Cell2).
    fn focused(&self) -> Style;

    /// The appearance when something is disabled of the
    /// [`Cell2`](crate::widget::cell2::Cell2).
    fn disabled(&self) -> Style;
}

/// The state of the style
#[derive(Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum StyleState {
    /// Use the active style
    Active,
    /// Use the selected style
    Selected,
    /// Use the hovered style
    Hovered,
    /// Use the focused style
    Focused,
    /// Use the disabled style
    Disabled,
}

/// The default appearance of the [`Cell2`](crate::widget::cell2::Cell2).
#[derive(Clone, Copy, Debug)]
pub struct Default;

impl StyleSheet for Default {
    fn active(&self) -> Style {
        Style {
            shadow_offset: Vector::new(0.0, 1.0),
            background: Background::Color([0.87, 0.87, 0.87].into()),
            border_radius: 0.0,
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
            label_background: None,
            text_color: Color::BLACK,

            overlay_shadow_offset: Vector::new(0.0, 1.0),
            overlay_background: Background::Color([0.87, 0.87, 0.87].into()),
            overlay_border_radius: 0.0,
            overlay_border_width: 1.0,
            overlay_border_color: [0.0, 0.0, 0.0, 0.5].into(),
            overlay_label_background: None,
            overlay_text_color: Color::BLACK,

            separator_radius: 5.0,
            separator_width: 1.0,
            separator_color: [0.7, 0.7, 0.7].into(),
            separator_horizontal_margin: 1.0,
        }
    }

    fn selected(&self) -> Style {
        Style {
            background: Background::Color([0.4, 0.4, 0.8].into()),
            ..self.active()
        }
    }

    fn hovered(&self) -> Style {
        let active = self.active();

        Style {
            label_background: Some(Background::Color([0.4, 0.4, 0.8].into())),
            //background: Background::Color([0.4, 0.4, 0.8].into()),
            overlay_label_background: Some(Background::Color([0.4, 0.4, 0.8].into())),

            ..active
        }
    }

    fn focused(&self) -> Style {
        Style { ..self.active() }
    }

    fn disabled(&self) -> Style {
        Style {
            text_color: [0.0, 0.0, 0.0, 0.75].into(),
            overlay_text_color: [0.0, 0.0, 0.0, 0.75].into(),
            ..self.active()
        }
    }
}

#[allow(clippy::use_self)]
impl std::default::Default for Box<dyn StyleSheet> {
    fn default() -> Self {
        Box::new(Default)
    }
}

#[allow(clippy::use_self)]
impl<T> From<T> for Box<dyn StyleSheet>
where
    T: 'static + StyleSheet,
{
    fn from(style: T) -> Self {
        Box::new(style)
    }
}
