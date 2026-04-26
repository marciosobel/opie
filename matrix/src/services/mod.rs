pub mod authenticate;
pub use authenticate::{authenticate, is_authenticated, login, restore_session, save_session};

pub mod client;
pub use client::{ClientBuildErrorKind, new_client, new_client_with_session};

pub mod room;
pub use room::{Room, list_joined_rooms};

pub mod timeline;
pub use timeline::{Timeline, TimelineEvent, timeline};

pub mod user;
pub use user::UserInfo;

pub mod sas_verification;
pub use sas_verification::Action as SasAction;

pub mod device;
pub use device::{Device, DeviceKind};
