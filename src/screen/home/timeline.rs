use std::sync::Arc;

use matrix::services::timeline;

#[derive(Debug, Clone)]
pub struct Timeline {
    pub items: timeline::Vector<Arc<timeline::TimelineItem>>,
    pub hit_start: bool,
    pub hit_end: bool,
}

impl Timeline {
    pub fn new() -> Self {
        Self {
            items: timeline::Vector::new(),
            hit_end: false,
            hit_start: false,
        }
    }

    pub fn with_items(items: timeline::Vector<Arc<timeline::TimelineItem>>) -> Self {
        Self {
            items,
            hit_end: false,
            hit_start: false,
        }
    }
}
