use iced::{
    Color, Element, Length, Point, Renderer, Size, Theme, mouse,
    widget::{canvas, container, image, stack, text},
};

pub enum BackgroundKind {
    Image(image::Handle),
    Color(Color),
}

struct Solid(Color);

impl<Message> canvas::Program<Message> for Solid {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: iced::Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry<Renderer>> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        frame.fill_rectangle(
            Point::ORIGIN,
            Size::new(bounds.width, bounds.height),
            self.0,
        );

        vec![frame.into_geometry()]
    }
}

pub fn background<'a, Message: 'a>(kind: &'a BackgroundKind) -> Element<'a, Message> {
    match kind {
        BackgroundKind::Color(c) => canvas(Solid(*c))
            .width(Length::Fill)
            .height(Length::Fill)
            .into(),
        BackgroundKind::Image(i) => stack![
            image(i).width(Length::Fill).height(Length::Fill),
            // TODO; finish credits
            container(text("Photo, John Doe, Unsplash"))
                .align_left(Length::Fill)
                .align_bottom(Length::Fill)
                .padding(15)
        ]
        .into(),
    }
}
