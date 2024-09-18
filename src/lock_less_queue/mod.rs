use node::Node;
use std::sync::{
    atomic::{AtomicPtr, Ordering},
    Arc,
};

pub mod node;

pub struct Queue {
    head: AtomicPtr<Node>,
    tail: AtomicPtr<Node>,
}

impl Queue {
    pub fn new(default_val: String) -> Self {
        let node = Node {
            value: default_val,
            next: None,
        };

        let node_arc_pointer = Arc::into_raw(Arc::new(node)) as *mut Node;

        Queue {
            head: AtomicPtr::new(node_arc_pointer),
            tail: AtomicPtr::new(node_arc_pointer),
        }
    }

    pub fn enqueue(&self, item: String) {
        let node_arc = Arc::new(Node::new(item));

        loop {
            let tail_loaded = self.tail.load(Ordering::SeqCst);

            let mut tail_node = get_clone_value(tail_loaded);

            match tail_node.next {
                None => {
                    tail_node.next = Some(node_arc.clone());
                    let tail_node_arc = Arc::new(tail_node);

                    match self.tail.compare_exchange(
                        tail_loaded,
                        Arc::into_raw(tail_node_arc.clone()) as *mut Node,
                        Ordering::SeqCst,
                        Ordering::SeqCst,
                    ) {
                        Ok(_) => {
                            self.tail.compare_exchange(
                                tail_loaded,
                                Arc::into_raw(tail_node_arc.as_ref().next.as_ref().unwrap().clone())
                                    as *mut Node,
                                Ordering::SeqCst,
                                Ordering::SeqCst,
                            );
                            break;
                        }
                        Err(_) => {}
                    }
                }
                Some(node_arc) => {
                    _ = self.tail.compare_exchange(
                        tail_loaded,
                        Arc::into_raw(node_arc) as *mut Node,
                        Ordering::SeqCst,
                        Ordering::SeqCst,
                    );
                }
            }
        }
    }

    pub fn dequeue(&self) -> Option<String> {
        loop {
            let head_loaded = self.head.load(Ordering::SeqCst);
            let tail_loaded = self.tail.load(Ordering::SeqCst);

            let head_node = Arc::new(get_clone_value(head_loaded));
            let tail_node = get_clone_value(tail_loaded);

            if head_loaded.eq(&tail_loaded) {
                match &head_node.next {
                    None => return None,
                    Some(node) => {
                        _ = self.tail.compare_exchange(
                            tail_loaded,
                            Arc::into_raw(node.into()) as *mut Node,
                            Ordering::SeqCst,
                            Ordering::SeqCst,
                        );
                        continue;
                    }
                }
            }

            let next = head_node.as_ref().next.as_ref().clone();
            match self.head.compare_exchange(
                head_loaded,
                Arc::into_raw(head_node.as_ref().next.as_ref().unwrap().into()) as *mut Node,
                Ordering::SeqCst,
                Ordering::SeqCst,
            ) {
                Ok(_) => {
                    return Some(next.unwrap().value.clone());
                }
                Err(_) => {}
            }
        }
    }
}

fn get_clone_value<T>(ptr: *mut T) -> T
where
    T: Clone,
{
    let ptr_arc = unsafe { Arc::from_raw(ptr) };
    (*ptr_arc).clone()
}
