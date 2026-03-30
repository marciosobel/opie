use iced::Settings;
use lucide_icons::LUCIDE_FONT_BYTES;
use opie::App;
use tracing_subscriber::EnvFilter;

fn main() -> iced::Result {
    let env_filter = EnvFilter::from_default_env().add_directive("opie=info".parse().unwrap());

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .try_init()
        .unwrap();

    tracing::info!("Initializing daemon");

    iced::daemon(App::new, App::update, App::view)
        .settings(Settings {
            fonts: vec![LUCIDE_FONT_BYTES.into()],
            ..Default::default()
        })
        .subscription(App::subscription)
        .theme(App::theme)
        .title("Opie")
        .run()
}
