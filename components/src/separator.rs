use iced::{
    widget::container::{Container, Style},
    Element, Length, Theme,
};

/// Creates a horizontal separator. Useful for spacing list items.
///
/// This is the equivalent to:
/// ```rust,no_run
/// let horizontal = space().height(1).width(Length::Fill);
/// ```
pub fn horizontal<'a, Message>() -> Separator<'a, Message>
where
    Message: Clone + 'a,
{
    Separator::new(Orientation::Horizontal)
}

/// Creates a vertical separator. Useful for spacing list items.
///
/// This is the equivalent to:
/// ```rust,no_run
/// # use iced::widget::space;
/// # use iced::Length;
/// let vertical = space().height(Length::Fill).width(1);
/// ```
pub fn vertical<'a, Message>() -> Separator<'a, Message>
where
    Message: Clone + 'a,
{
    Separator::new(Orientation::Vertical)
}
pub enum Orientation {
    Horizontal,
    Vertical,
}

pub struct Separator<'a, Message>
where
    Message: Clone + 'a,
{
    orientation: Orientation,
    size: Length,
    thickness: Length,
    container: Container<'a, Message>,
}

impl<'a, Message> Separator<'a, Message>
where
    Message: Clone + 'a,
{
    /// Creates a separator using the provided orientation. Useful for spacing list items.
    ///
    /// This is the equivalent to creating an empty container with size 1.
    pub fn new(orientation: Orientation) -> Self {
        let container = Container::new(iced::widget::space()).style(|theme: &Theme| {
            let palette = theme.extended_palette();
            Style::default().background(palette.background.weak.color)
        });

        Self {
            orientation,
            size: Length::Fill,
            thickness: Length::Fixed(1.0),
            container,
        }
    }

    /// Changes the size of the separator. Defaults to [`Length::Fill`].
    pub fn size(mut self, size: impl Into<Length>) -> Self {
        self.size = size.into();
        self
    }

    /// Changes the thickness of the separator. Defaults to [`Length::Fixed(1.0)`].
    pub fn thickness(mut self, thickness: impl Into<Length>) -> Self {
        self.thickness = thickness.into();
        self
    }

    /// Sets the style of the [`Separator`].
    pub fn style(mut self, style: impl Fn(&Theme) -> Style + 'a) -> Self {
        self.container = self.container.style(style);
        self
    }

    fn into_element(self) -> Element<'a, Message> {
        let (width, height) = match self.orientation {
            Orientation::Horizontal => (self.size, self.thickness),
            Orientation::Vertical => (self.thickness, self.size),
        };

        self.container.width(width).height(height).into()
    }
}

impl<'a, Message> Into<Element<'a, Message>> for Separator<'a, Message>
where
    Message: Clone + 'a,
{
    fn into(self) -> Element<'a, Message> {
        self.into_element()
    }
}
