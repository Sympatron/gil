use crate::{atomic::Ordering, hint, mpsc::queue::QueuePtr};

#[derive(Clone)]
pub struct Sender<T> {
    ptr: QueuePtr<T>,
    local_reserved: usize,
    local_tail: usize,
}

impl<T> Sender<T> {
    pub(crate) fn new(queue_ptr: QueuePtr<T>) -> Self {
        Self {
            ptr: queue_ptr,
            local_reserved: 0,
            local_tail: 0,
        }
    }

    pub fn send(&mut self, value: T) {
        let mut new_tail = self.local_reserved + 1;

        loop {
            while new_tail > self.ptr.head().load(Ordering::Acquire) + self.ptr.size {
                hint::spin_loop();
            }

            match self.ptr.reserved().compare_exchange_weak(
                self.local_reserved,
                new_tail,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Err(cur_reserved) => self.local_reserved = cur_reserved,
                Ok(_) => break,
            }

            new_tail = self.local_reserved + 1;

            hint::spin_loop();
        }

        unsafe { self.ptr.set(self.local_reserved, value) };
        loop {
            match self.ptr.tail().compare_exchange_weak(
                new_tail - 1,
                new_tail,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => break,
                Err(cur_tail) => self.local_tail = cur_tail,
            }
            hint::spin_loop();
        }

        self.local_reserved = new_tail;
        self.local_tail = new_tail;
    }

    pub fn try_send(&mut self, value: T) -> Result<(), T> {
        let mut new_tail = self.local_reserved + 1;

        loop {
            if new_tail > self.ptr.head().load(Ordering::Acquire) + self.ptr.size {
                return Err(value);
            }

            match self.ptr.reserved().compare_exchange_weak(
                self.local_reserved,
                new_tail,
                Ordering::AcqRel,
                Ordering::Relaxed,
            ) {
                Err(cur_reserved) => self.local_reserved = cur_reserved,
                Ok(_) => break,
            }

            new_tail = self.local_reserved + 1;

            hint::spin_loop();
        }

        unsafe { self.ptr.set(self.local_reserved, value) };
        loop {
            match self.ptr.tail().compare_exchange_weak(
                new_tail - 1,
                new_tail,
                Ordering::AcqRel,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(cur_tail) => self.local_tail = cur_tail,
            }
            hint::spin_loop();
        }

        self.local_reserved = new_tail;
        self.local_tail = new_tail;

        Ok(())
    }
}

unsafe impl<T: Send> Send for Sender<T> {}
