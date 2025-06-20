use iced::{
    Color, Element, Length, Theme,
    widget::{Svg, button, svg},
};
use rust_embed::Embed;

#[derive(Embed)]
#[folder = "icons/"]
#[prefix = "icons/"]
struct Icon;

pub fn icon<'a>(path: &str) -> Svg<'a, iced::Theme> {
    let bytes = Icon::get(path).unwrap().data;

    svg(svg::Handle::from_memory(bytes))
        .height(Length::Fixed(16.0))
        .width(Length::Fixed(16.0))
        .style(white)
}

pub fn icon_button<'a, Message: 'a + Clone>(
    handle: &str,
    on_press: Message,
) -> Element<'a, Message> {
    button(icon(handle))
        .style(button::text)
        .on_press(on_press)
        .into()
}

fn white(_theme: &Theme, _status: svg::Status) -> svg::Style {
    svg::Style {
        color: Some(Color::WHITE),
    }
}
