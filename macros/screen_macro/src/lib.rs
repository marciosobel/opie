/// A macro to reduce boilerplate when matching on the current screen in an app.
/// It allows you to easily extract the state of a specific screen without having to write out the full `let ... else` statement each time.
///
/// # Example
/// ```rust
/// use screen_macro::screen;
///
/// mod auth {
///     pub struct State;
/// }
///
/// enum Message {
///     Auth(auth::Message),
/// }
///
/// enum Screen {
///     Auth(auth::State),
/// }
///
/// struct App {
///     screen: Screen,
/// }
///
/// impl App {
///     fn update(&mut self, message: Message) -> iced::Task<Message> {
///         match message {
///             Message::Auth(message) => {
///                 // If the current screen is not `Screen::Auth`,
///                 // this returns `iced::Task::none()`.
///                 let auth = screen!(self, Screen::Auth);
///
///                 // Now you can use `auth` as the state of the `Auth` screen.
///                 let action = auth.update(message);
///             }
///         }
///     }
/// }
/// ```
#[macro_export]
macro_rules! screen {
    ( $self:ident, $screen:path ) => {
        match &mut $self.screen {
            $screen(x) => x,
            _ => return iced::task::Task::none(),
        }
    };
}
