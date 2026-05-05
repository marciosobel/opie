use std::collections::HashMap;

use matrix::services::room::RoomId;

#[derive(Debug, Clone, Default)]
pub struct Collapsibles {
    pub spaces: HashMap<RoomId, bool>,
    pub dms: bool,
    pub groups: bool,
}
