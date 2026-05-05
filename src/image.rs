use iced::widget::image;

#[derive(Debug, Clone)]
pub enum Image {
    Ready(image::Handle),
    Fetching,
    None,
}
