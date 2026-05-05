use std::collections::HashMap;

use matrix::services::{room::RoomId, timeline::EventId};

use crate::Image;

#[derive(Debug, Clone, Default)]
pub struct ImageCache {
    pub rooms: HashMap<RoomId, Image>,
    pub timeline: HashMap<EventId, Image>,
}
