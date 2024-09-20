use node::Node;
use std::ptr;
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
    head: AtomicPtr<Option<Node>>,
    tail: AtomicPtr<Option<Node>>,
}
// TODO: in future , extend the struct with a length u32 object

impl Queue {
    pub fn new(default_val: String) -> Self {
        let node = Node::new(default_val);

        let node_arc_pointer = Arc::into_raw(Arc::new(Some(node))) as *mut Option<Node>;

        Queue {
            head: AtomicPtr::new(node_arc_pointer),
            tail: AtomicPtr::new(node_arc_pointer),
        }
    }

    pub fn enqueue(&self, item: String) {
        let node_arc = Arc::new(Some(Node::new(item)));

        loop {
            let tail_ptr = self.tail.load(Ordering::SeqCst);
            let tail_ref = unsafe { Arc::from_raw(tail_ptr) };
            // TODO: why the fuck ????
            let tail_ref_clone_unwrapped = tail_ref.as_ref().as_ref().unwrap();

            let next_ptr = tail_ref_clone_unwrapped.next.load(Ordering::SeqCst);
            // TODO: do I really need this clone ? what happens if i don't use clone ,
            // will there be some sort of memory problem ?
            let new_next_ptr = Arc::into_raw(node_arc.clone()) as *mut _;

            // DoneTODO: is it possible to not clone the object but rather modify the member value ?
            // no, because that will require locking, since the object is pointed by an atomic
            // pointer not the member

            // TODO: seems like ownership is not transferred in case of *mut T. Investigate further
            // into rust's black magic powder.
            match next_ptr.is_null() {
                true => {
                    match tail_ref_clone_unwrapped.next.compare_exchange(
                        next_ptr,
                        new_next_ptr,
                        Ordering::SeqCst,
                        Ordering::SeqCst,
                    ) {
                        Ok(_) => {
                            // NOTE: this operation of setting the queue's tail as the next value
                            // is done after the loop in the paper's pseudocode, But there
                            // shouldn't be any difference in operations due to this re-ordering
                            // TODO: lesser .as_ref to be used here
                            self.tail.compare_exchange(
                                tail_ptr,
                                new_next_ptr,
                                Ordering::SeqCst,
                                Ordering::SeqCst,
                            );
                            consume_arc_variable(tail_ref);
                            // TODO: do any needed cleanup operations
                            break;
                        }
                        Err(_) => {}
                    }
                }
                false => {
                    _ = self.tail.compare_exchange(
                        tail_ptr,
                        next_ptr,
                        Ordering::SeqCst,
                        Ordering::SeqCst,
                    );
                }
            }
            consume_arc_variable(tail_ref);
        }
    }

    pub fn dequeue(&self) -> Option<String> {
        loop {
            let head_ptr = self.head.load(Ordering::SeqCst);
            let tail_ptr = self.tail.load(Ordering::SeqCst);

            let head_ref = unsafe { Arc::from_raw(head_ptr) };

            let head_next_ptr = head_ref
                .as_ref()
                .as_ref()
                .unwrap()
                .next
                .load(Ordering::SeqCst);

            if head_ptr.eq(&tail_ptr) {
                match head_next_ptr.is_null() {
                    true => return None,
                    false => {
                        _ = self.tail.compare_exchange(
                            tail_ptr,
                            // TODO: why and what is into needed for ?
                            head_next_ptr,
                            Ordering::SeqCst,
                            Ordering::SeqCst,
                        );
                        consume_arc_variable(head_ref);
                        continue;
                    }
                }
            }

            let value = head_ref.as_ref().as_ref().unwrap().value.clone();
            match self.head.compare_exchange(
                head_ptr,
                head_next_ptr,
                Ordering::SeqCst,
                Ordering::SeqCst,
            ) {
                Ok(_) => {
                    consume_arc_variable(head_ref);
                    return Some(value);
                }
                Err(_) => {}
            }

            consume_arc_variable(head_ref);
        }
    }
}

fn consume_arc_variable<T>(ptr: Arc<T>) {
    let _ = Arc::into_raw(ptr);
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
