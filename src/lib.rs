use lock_less_queue::Queue;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

pub mod lock_less_queue;

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    // TODO: write down lyrics / copy it of Dua lipa Levitating, read from a file and print it out
    // thread by thread in parallel
    #[test]
    fn test_multi_threaded_queue() {
        let queue = Arc::new(Queue::new("Yo Dua Lipa".to_owned()));

        // TODO: issue here is that the pogram / test runs successfully when a thread performs only
        // one operation. If there are more than one operations , then i get a use-after-heap-free
        // error
        let handles = (0..2)
            .map(|j| {
                let queue_clone = queue.clone();
                std::thread::spawn(move || {
                    for i in 0..2 {
                        queue_clone.enqueue(format!("{}", j).to_owned());
                    }
                })
            })
            .collect::<Vec<_>>();

        for handle in handles {
            handle.join();
        }
    }
}
