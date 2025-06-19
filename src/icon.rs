use iced::{
    Color, Element, Length, Theme,
    widget::{button, svg},
};

pub fn icon<'a, Message>(handle: impl Into<svg::Handle>) -> Element<'a, Message> {
    svg(handle)
        .height(Length::Fixed(16.0))
        .width(Length::Fixed(16.0))
        .style(white)
        .into()
}

pub fn icon_button<'a, Message: 'a + Clone>(
    handle: impl Into<svg::Handle>,
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
