use node::Node;
use std::sync::{
    atomic::{AtomicPtr, Ordering},
    Arc,
};

pub mod node;

// NOTE: Design decisions : so, There is an Arc outside of AtomicPtr here because I need to share
// the same node between head and tail some times.
// Obviously , there can't be two atomic pointer objects pointing to the same node. It's ensured by
// the rust compiler itself, since AtomicPtr::new takes *mut T.
// huhuh, the joke's on me. There can be !!!
// and not using Arc for the same reason because head and tail will be different many times.
//
// TODO: is there a need of public keyword here ?
pub struct Queue {
    head: AtomicPtr<Node>,
    tail: AtomicPtr<Node>,
}
// TODO: in future , extend the struct with a length u32 object

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
            // DoneTODO: is it possible to not clone the object but rather modify the member value ?
            // no, because that will require locking, since the object is pointed by an atomic
            // pointer not the member

            // TODO: seems like ownership is not transferred in case of *mut T. Investigate further
            // into rust's black magic powder.
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
                            // NOTE: this operation of setting the queue's tail as the next value
                            // is done after the loop in the paper's pseudocode, But there
                            // shouldn't be any difference in operations due to this re-ordering
                            // TODO: lesser .as_ref to be used here
                            self.tail.compare_exchange(
                                tail_loaded,
                                Arc::into_raw(tail_node_arc.as_ref().next.as_ref().unwrap().clone())
                                    as *mut Node,
                                Ordering::SeqCst,
                                Ordering::SeqCst,
                            );
                            // TODO: do any needed cleanup operations
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
                            // TODO: why and what is into needed for ?
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
                // TODO: why is as_ref needed here?
                Arc::into_raw(head_node.as_ref().next.as_ref().unwrap().into()) as *mut Node,
                Ordering::SeqCst,
                Ordering::SeqCst,
            ) {
                Ok(_) => {
                    // TODO: free the head pointer;
                    // TODO: check : can I avoid this clone here ?
                    return Some(next.unwrap().value.clone());
                    // TODO: why is unwrap needed over here ?
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
    let cloned_val = (*ptr_arc).clone();
    let _ = Arc::into_raw(ptr_arc);
    cloned_val
}
