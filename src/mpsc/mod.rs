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
        let (tx, mut rx) = channel(NonZeroUsize::new(8).unwrap());

        thread::scope(move |scope| {
            for j in 0..10 {
                let mut tx = tx.clone();
                scope.spawn(move || {
                    for i in 0..10 {
                        println!("{j}: sending {i}");
                        tx.send(i);
                        println!("{j}: sent {i}");
                    }
                });
            }

            let mut sum = 0;
            for _ in 0..10 {
                for _ in 0..10 {
                    println!("recving");
                    sum += rx.recv();
                    println!("recved");
                }
            }

            assert_eq!(sum, (10 + (10 + 1)) / 2 * 10);
        });
    }
}
