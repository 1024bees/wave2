use iced_native::Color;

//TODO: make this a const fn??
pub fn color_from_str(color: &str) -> Color {
    let mut chars = color.chars();

    let red: u8 = u8::from_str_radix(&color[1..3], 16).unwrap();
    let green: u8 = u8::from_str_radix(&color[3..5], 16).unwrap();
    let blue: u8 = u8::from_str_radix(&color[5..7], 16).unwrap();

    Color::from_rgb8(red, green, blue)
}

//
//
//
mod tests {
    use super::*;
    #[test]
    fn gray_str() {
        assert_eq!(
            color_from_str("#797986"),
            Color::from_rgb8(0x79, 0x79, 0x86),
            "Colors arent equal!"
        );
    }
}
