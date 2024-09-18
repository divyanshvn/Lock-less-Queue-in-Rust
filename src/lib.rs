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

    // thread by thread in parallel
    #[test]
    fn test_multi_threaded_queue() {
        let queue = Arc::new(Queue::new("Yo Dua Lipa".to_owned()));

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
