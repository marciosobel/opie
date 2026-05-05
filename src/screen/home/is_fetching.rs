use std::collections::HashSet;

use matrix::services::user::UserId;

#[derive(Debug, Clone)]
pub struct IsFetching {
    pub users: HashSet<UserId>,
    pub rooms: bool,
}

impl IsFetching {
    pub fn new() -> Self {
        Self {
            users: HashSet::new(),
            rooms: true,
        }
    }
}
