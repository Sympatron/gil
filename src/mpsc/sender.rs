use crate::{atomic::Ordering, hint, queue::QueuePtr};

#[derive(Clone)]
pub struct Sender<T> {
    ptr: QueuePtr<T>,
    local_tail: usize,
}

impl<T> Sender<T> {
    pub(crate) fn new(queue_ptr: QueuePtr<T>) -> Self {
        Self {
            ptr: queue_ptr,
            local_tail: 0,
        }
    }

    pub fn send(&mut self, value: T) {
        let mut new_tail = self.local_tail + 1;

        loop {
            while new_tail == self.ptr.head().load(Ordering::Acquire) {
                hint::spin_loop();
            }
            match self.ptr.tail().compare_exchange_weak(
                self.local_tail,
                new_tail,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Err(cur_tail) => self.local_tail = cur_tail,
                Ok(_) => break,
            }

            new_tail = self.local_tail + 1;
            hint::spin_loop();
        }

        unsafe { self.ptr.set(self.local_tail, value) };
        self.local_tail = new_tail;
    }
}

unsafe impl<T: Send> Send for Sender<T> {}
