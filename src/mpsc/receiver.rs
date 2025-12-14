use crate::{atomic::Ordering, hint, mpsc::queue::QueuePtr};

/// The consumer end of the queue.
///
/// This struct is `Send` but not `Sync`. It can be moved to another thread, but cannot be shared
/// across threads.
pub struct Receiver<T> {
    ptr: QueuePtr<T>,
    local_tail: usize,
    local_head: usize,
}

impl<T> Receiver<T> {
    pub(crate) fn new(queue_ptr: QueuePtr<T>) -> Self {
        Self {
            ptr: queue_ptr,
            local_tail: 0,
            local_head: 0,
        }
    }

    pub fn recv(&mut self) -> T {
        while self.local_head == self.local_tail {
            hint::spin_loop();
            self.load_tail();
        }

        // SAFETY: head != tail which means queue is not empty and head has valid initialised
        //         value
        let ret = unsafe { self.ptr.get(self.local_head) };
        let new_head = self.local_head + 1;
        self.store_head(new_head);
        self.local_head = new_head;

        ret
    }

    pub fn try_recv(&mut self) -> Option<T> {
        if self.local_head == self.local_tail {
            self.load_tail();
            if self.local_head == self.local_tail {
                return None;
            }
        }

        // SAFETY: head != tail which means queue is not empty and head has valid initialised
        //         value
        let ret = unsafe { self.ptr.get(self.local_head) };
        let new_head = self.local_head + 1;
        self.store_head(new_head);
        self.local_head = new_head;

        Some(ret)
    }

    #[inline(always)]
    fn store_head(&self, value: usize) {
        self.ptr.head().store(value, Ordering::Release);
    }

    #[inline(always)]
    fn load_tail(&mut self) {
        self.local_tail = self.ptr.tail().load(Ordering::Acquire);
    }
}

unsafe impl<T: Send> Send for Receiver<T> {}
