use lucide_icons::Icon;
use matrix_sdk::{encryption::identities::Device as MatrixDevice, ruma::OwnedDeviceId};

#[derive(Debug, Clone)]
pub struct Device {
    pub id: OwnedDeviceId,
    pub display_name: Option<String>,
    pub verified: bool,
    pub kind: DeviceKind,
}

#[derive(Debug, Clone, Default)]
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
