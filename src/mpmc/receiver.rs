use crate::{atomic::Ordering, hint, mpmc::queue::QueuePtr, thread};

#[derive(Clone)]
pub struct Receiver<T> {
    ptr: QueuePtr<T>,
    local_head: usize,
}

impl<T> Receiver<T> {
    pub(crate) fn new(queue_ptr: QueuePtr<T>) -> Self {
        Self {
            ptr: queue_ptr,
            local_head: 0,
        }
    }

    pub fn recv(&mut self) -> T {
        let head = self.ptr.head().fetch_add(1, Ordering::Relaxed);
        let next = head.wrapping_add(1);
        self.local_head = next;

        let cell = self.ptr.at(head);
        let mut spin_count = 0;
        while cell.epoch().load(Ordering::Acquire) != next {
            if spin_count < 128 {
                hint::spin_loop();
                spin_count += 1;
            } else {
                thread::yield_now();
            }
        }

        let ret = unsafe { cell.get() };
        cell.epoch()
            .store(head + self.ptr.capacity, Ordering::Release);

        ret
    }

    pub fn try_recv(&mut self) -> Option<T> {
        todo!()
    }
}

unsafe impl<T: Send> Send for Receiver<T> {}
