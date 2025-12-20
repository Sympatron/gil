use std::{num::NonZeroUsize, ptr::NonNull};

use crate::spsc;

mod receiver;
mod sender;

pub fn channel<T>(
    max_shards: NonZeroUsize,
    capacity_per_shard: NonZeroUsize,
) -> (sender::Sender<T>, receiver::Receiver<T>) {
    debug_assert_ne!(max_shards.get(), 0, "number of shards must be > 0");
    debug_assert!(
        max_shards.is_power_of_two(),
        "number of shards must be a power of 2"
    );

    let mut shards = Box::new_uninit_slice(max_shards.get());
    for i in 0..max_shards.get() {
        shards[i].write(spsc::QueuePtr::<T>::with_size(capacity_per_shard));
    }
    // SAFETY: Box::new was valid
    let shards = unsafe { NonNull::new_unchecked(Box::into_raw(shards)).cast() };

    // SAFETY: Sender::init(..) will clone, while Receiver::new(..) will move
    (sender::Sender::new(shards, max_shards), unsafe {
        receiver::Receiver::new(shards, max_shards.get())
    })
}

#[cfg(all(test, not(feature = "loom")))]
mod test {
    use super::*;

    use crate::thread;

    #[test]
    fn basic() {
        const THREADS: u32 = 8;
        const ITER: u32 = 10;

        let (mut tx, mut rx) = channel(
            NonZeroUsize::new(THREADS as usize).unwrap(),
            NonZeroUsize::new(4).unwrap(),
        );

        thread::scope(move |scope| {
            for thread_id in 0..THREADS - 1 {
                let mut tx = tx.clone().unwrap();
                scope.spawn(move || {
                    for i in 0..ITER {
                        tx.send((thread_id, i));
                    }
                });
            }
            scope.spawn(move || {
                for i in 0..ITER {
                    tx.send((THREADS - 1, i));
                }
            });

            let mut sum = 0;
            for _ in 0..THREADS {
                for _ in 0..ITER {
                    let (_thread_id, i) = rx.recv();
                    sum += i;
                }
            }

            assert_eq!(sum, (ITER * (ITER - 1)) / 2 * THREADS);
        });
    }

    #[test]
    fn test_valid_try_sends() {
        let (mut tx, mut rx) =
            channel::<usize>(NonZeroUsize::new(1).unwrap(), NonZeroUsize::new(4).unwrap());
        for _ in 0..4 {
            assert!(rx.try_recv().is_none());
        }
        for i in 0..4 {
            tx.try_send(i).unwrap();
        }
        assert!(tx.try_send(5).is_err());

        for i in 0..4 {
            assert_eq!(rx.try_recv(), Some(i));
        }
        assert!(rx.try_recv().is_none());
        for i in 0..4 {
            tx.try_send(i).unwrap();
        }
    }

    #[test]
    fn test_drop_full_capacity() {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicUsize, Ordering};

        struct DropCounter(Arc<AtomicUsize>);

        impl Drop for DropCounter {
            fn drop(&mut self) {
                self.0.fetch_add(1, Ordering::SeqCst);
            }
        }

        let dropped_count = Arc::new(AtomicUsize::new(0));

        {
            let (mut tx, _rx) = channel::<DropCounter>(
                NonZeroUsize::new(1).unwrap(),
                NonZeroUsize::new(4).unwrap(),
            );

            for _ in 0..4 {
                tx.send(DropCounter(dropped_count.clone()));
            }
        }

        let count = dropped_count.load(Ordering::SeqCst);
        assert_eq!(
            count, 4,
            "Expected 4 items to be dropped, but got {}",
            count
        );
    }
}
