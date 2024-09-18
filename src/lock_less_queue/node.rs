use std::sync::Arc;

#[derive(Clone)]
pub struct Node {
    pub value: String,
    pub next: Option<Arc<Node>>,
}

impl Node {
    pub fn new(item: String) -> Self {
        Node {
            value: item,
            next: None,
        }
    }
}
