use iced::widget::{center, text};

fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .theme(iced::theme::Theme::Dark)
        .run()
}

struct App;

#[derive(Debug, Clone)]
enum Message {}

impl App {
    fn new() -> Self {
        Self
    }

    fn update(&mut self, _message: Message) {}

    fn view(&self) -> iced::Element<'_, Message> {
        return center(text("Hello, opie!")).into();
    }
}
