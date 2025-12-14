use crate::{atomic::Ordering, hint, queue::QueuePtr};

#[derive(Clone)]
pub struct Sender<T> {
    ptr: QueuePtr<T>,
    local_head: usize,
    local_tail: usize,
}

impl<T> Sender<T> {
    pub(crate) fn new(queue_ptr: QueuePtr<T>) -> Self {
        Self {
            ptr: queue_ptr,
            local_head: 0,
            local_tail: 0,
        }
    }

    #[inline(always)]
    fn next_tail(&self) -> usize {
        let next = self.local_tail + 1;
        if next == self.ptr.capacity { 0 } else { next }
    }

    pub fn send(&mut self, value: T) {
        let mut new_tail = self.next_tail();

        loop {
            while new_tail == self.local_head {
                hint::spin_loop();
                self.load_head();
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

            new_tail = self.next_tail();
            hint::spin_loop();
        }

        self.ptr.set(self.local_tail, value);
        self.local_tail = new_tail;
    }

    #[inline(always)]
    fn load_head(&mut self) {
        self.local_head = self.ptr.head().load(Ordering::Acquire);
    }
}

unsafe impl<T: Send> Send for Sender<T> {}
