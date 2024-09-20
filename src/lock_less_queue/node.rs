use std::{ptr, sync::atomic::AtomicPtr};

// TODO: replace String with a generic type after successful phase 1
pub struct Node {
    pub value: String,
    pub next: AtomicPtr<Option<Node>>,
}

impl Node {
    pub fn new(item: String) -> Self {
        Node {
            value: item,
            next: AtomicPtr::new(ptr::null_mut()) as AtomicPtr<Option<Node>>,
        }
    }
}
