use lucide_icons::Icon;
use matrix_sdk::{
    encryption::identities::Device as MatrixDevice, ruma::MilliSecondsSinceUnixEpoch,
};

pub use matrix_sdk::ruma::OwnedDeviceId as DeviceId;

/// A wrapper with useful device info
#[derive(Debug, Clone)]
pub struct Device {
    /// The ID of the device
    pub id: DeviceId,
    /// The name of the device
    pub display_name: Option<String>,
    /// Whether the device is verified or not
    pub verified: bool,
    /// The kind of the device
    pub kind: DeviceKind,
    /// Whether or not this device is the the one running this app
    pub is_self: bool,
    /// Last time this device has been active
    pub last_seen: Option<MilliSecondsSinceUnixEpoch>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum DeviceKind {
    Desktop,
    Web,
    Mobile,
    #[default]
    Unknown,
}

impl Device {
    pub fn new(device: MatrixDevice) -> Self {
        let id = device.device_id().to_owned();
        let display_name = device.display_name().map(String::from);
        let verified = device.is_verified_with_cross_signing();
        let kind = match &display_name {
            None => DeviceKind::Unknown,
            Some(display_name) => {
                let name = display_name.to_lowercase();
                if name.contains("web") || name.contains("browser") {
                    DeviceKind::Web
                } else if name.contains("android")
                    || name.contains("ios")
                    || name.contains("iphone")
                {
                    DeviceKind::Mobile
                } else {
                    DeviceKind::Desktop
                }
            }
        };

        Self {
            id,
            display_name,
            verified,
            kind,
            is_self: false,
            last_seen: None,
        }
    }
}

impl DeviceKind {
    pub fn widget<'a>(&self) -> iced::widget::Text<'a> {
        self.icon().widget()
    }

    pub fn icon(&self) -> Icon {
        self.clone().into()
    }
}

impl PartialEq for Device {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Device {}

impl From<MatrixDevice> for Device {
    fn from(value: MatrixDevice) -> Self {
        Device::new(value)
    }
}

impl Into<Icon> for DeviceKind {
    fn into(self) -> Icon {
        match self {
            DeviceKind::Desktop => Icon::Monitor,
            DeviceKind::Web => Icon::AppWindowMac,
            DeviceKind::Mobile => Icon::Smartphone,
            DeviceKind::Unknown => Icon::Box,
        }
    }
}
