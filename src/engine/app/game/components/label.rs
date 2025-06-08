use std::sync::atomic::{AtomicU32, Ordering};

pub struct Label{
    pub label: String,
    pub id: u32
}

static OBJECTS_NUM: AtomicU32 = AtomicU32::new(7);

impl Label {
    pub fn new(label: String) -> Self{
        let id = OBJECTS_NUM.fetch_add(1, Ordering::Relaxed);
        Self { label, id: id }
    }

    pub fn from_str(label: &str) -> Self{
        Self::new(label.to_string())
    }
}