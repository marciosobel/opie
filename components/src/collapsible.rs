use iced::{
    Alignment, Element, Theme,
    widget::{Button, button, column, row},
};

pub struct Collapsible<'a, Message> {
    toggler: Button<'a, Message>,
    content: Option<Element<'a, Message>>,
    on_open: Option<Message>,
    on_close: Option<Message>,
    open: bool,
    direction: Direction,
    spacing: u32,
    alignment: Alignment,
}

pub enum Direction {
    Vertical,
    Horizontal,
    VerticalReverse,
    HorizontalReverse,
}

/// Creates a collapsible element with a toggler and content. The toggler is a button that can be styled and will produce messages when pressed.
pub fn collapsible<'a, Message>(
    toggler: impl Into<Element<'a, Message>>,
) -> Collapsible<'a, Message>
where
    Message: Clone + 'a,
{
    Collapsible::new(toggler)
}

impl<'a, Message> Collapsible<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn new(toggler: impl Into<Element<'a, Message>>) -> Self {
        Self {
            toggler: button(toggler),
            content: None,
            on_open: None,
            on_close: None,
            open: false,
            direction: Direction::Vertical,
            spacing: 0,
            alignment: Alignment::Start,
        }
    }

    /// Sets the message that will be produced when the Collapsible is opened.
    pub fn on_open(mut self, message: Message) -> Self {
        self.on_open = Some(message);
        self
    }

    /// Sets the message that will be produced when the Collapsible is closed.
    pub fn on_close(mut self, message: Message) -> Self {
        self.on_close = Some(message);
        self
    }

    /// Sets whether the collapsible is opened.
    pub fn open(mut self, state: bool) -> Self {
        self.open = state;
        self
    }

    /// Sets the [`button::Style`] of the toggler.
    pub fn style(mut self, style: impl Fn(&Theme, button::Status) -> button::Style + 'a) -> Self {
        self.toggler = self.toggler.style(style);
        self
    }

    /// Sets the `width` of the toggler.
    pub fn width(mut self, width: impl Into<iced::Length>) -> Self {
        self.toggler = self.toggler.width(width);
        self
    }

    /// Sets the `height` of the toggler.
    pub fn height(mut self, height: impl Into<iced::Length>) -> Self {
        self.toggler = self.toggler.height(height);
        self
    }

    /// Sets the `padding` of the toggler.
    pub fn padding(mut self, padding: impl Into<iced::Padding>) -> Self {
        self.toggler = self.toggler.padding(padding);
        self
    }

    /// Sets whether the contents of the toggler should be clipped on overflow.
    pub fn clip(mut self, clip: bool) -> Self {
        self.toggler = self.toggler.clip(clip);
        self
    }

    /// Sets the alignment of the contents of the toggler.
    pub fn direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }

    /// Sets the alignment of the contents of the toggler to be [`Direction::Vertical`].
    pub fn vertical(mut self) -> Self {
        self.direction = Direction::Vertical;
        self
    }

    /// Sets the alignment of the contents of the toggler to be [`Direction::Horizontal`].
    pub fn horizontal(mut self) -> Self {
        self.direction = Direction::Horizontal;
        self
    }

    /// Sets the content of the Collapsible. This will be shown when the collapsible is opened.
    pub fn content(mut self, content: impl Into<Element<'a, Message>>) -> Self {
        self.content = Some(content.into());
        self
    }

    /// Sets the spacing between the toggler and the content.
    pub fn spacing(mut self, spacing: u32) -> Self {
        self.spacing = spacing;
        self
    }

    /// Sets the alignment between the toggler and the content.
    pub fn alignment(mut self, alignment: impl Into<iced::Alignment>) -> Self {
        self.alignment = alignment.into();
        self
    }

    fn render(self) -> Element<'a, Message> {
        let message = if self.open {
            self.on_close
        } else {
            self.on_open
        };

        let toggler = if let Some(message) = message {
            self.toggler.on_press(message)
        } else {
            self.toggler
        }
        .into();

        let Some(content) = self.content else {
            return toggler;
        };

        if !self.open {
            return toggler;
        }

        match self.direction {
            Direction::Vertical => column![toggler, content]
                .spacing(self.spacing)
                .align_x(self.alignment)
                .into(),
            Direction::Horizontal => row![toggler, content]
                .spacing(self.spacing)
                .align_y(self.alignment)
                .into(),
            Direction::VerticalReverse => column![content, toggler]
                .spacing(self.spacing)
                .align_x(self.alignment)
                .into(),
            Direction::HorizontalReverse => row![content, toggler]
                .spacing(self.spacing)
                .align_y(self.alignment)
                .into(),
        }
    }
}

impl<'a, Message> Into<Element<'a, Message>> for Collapsible<'a, Message>
where
    Message: Clone + 'a,
{
    fn into(self) -> Element<'a, Message> {
        self.render()
    }
}
