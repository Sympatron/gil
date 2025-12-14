use std::num::NonZeroUsize;

pub use self::{receiver::Receiver, sender::Sender};
use crate::queue::QueuePtr;

mod receiver;
mod sender;

pub fn channel<T>(capacity: NonZeroUsize) -> (Sender<T>, Receiver<T>) {
    let queue = QueuePtr::with_size(capacity);
    (Sender::new(queue.clone()), Receiver::new(queue))
}

#[cfg(all(test, not(feature = "loom")))]
mod test {
    use super::*;

    use crate::thread;

    #[test]
    fn basic() {
        const THREADS: u32 = 3;
        const ITER: u32 = 5;

        let (tx, mut rx) = channel(NonZeroUsize::new(8).unwrap());

        thread::scope(move |scope| {
            for thread_id in 0..THREADS {
                let mut tx = tx.clone();
                scope.spawn(move || {
                    for i in 0..ITER {
                        println!("{thread_id}: sending {i}");
                        tx.send((thread_id, i));
                        println!("{thread_id}: sent {i}");
                    }
                });
            }

            let mut sum = 0;
            for _ in 0..THREADS {
                for _ in 0..ITER {
                    let (thread_id, i) = rx.recv();
                    sum += i;
                    println!("recved: ({thread_id}:{i})");
                }
            }

            assert_eq!(sum, (ITER + (ITER + 1)) / 2 * THREADS);
        });
    }
}
