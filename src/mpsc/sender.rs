use std::ptr::NonNull;

use crate::{
    atomic::{AtomicUsize, Ordering},
    hint,
    mpsc::queue::QueuePtr,
};

#[derive(Clone)]
pub struct Sender<T> {
    ptr: QueuePtr<T>,
    reserved_ptr: NonNull<AtomicUsize>,
    local_reserved: usize,
    local_tail: usize,
}

impl<T> Sender<T> {
    pub(crate) fn new(queue_ptr: QueuePtr<T>) -> Self {
        let reserved_ptr = Box::into_raw(Box::new(AtomicUsize::new(0)));
        let reserved_ptr = unsafe { NonNull::new_unchecked(reserved_ptr) };
        Self {
            ptr: queue_ptr,
            reserved_ptr,
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
            match self.reserved().compare_exchange_weak(
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
        self.local_reserved = new_tail;
        loop {
            match self.ptr.tail().compare_exchange_weak(
                new_tail - 1,
                new_tail,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(new_tail) => {
                    self.local_tail = new_tail;
                    break;
                }
                Err(cur_tail) => {
                    self.local_tail = cur_tail;
                    if cur_tail >= new_tail {
                        self.local_reserved = cur_tail;
                        break;
                    }
                }
            }
            hint::spin_loop();
        }
    }

    #[inline(always)]
    fn reserved(&self) -> &AtomicUsize {
        unsafe { self.reserved_ptr.as_ref() }
    }
}

// FIXME: need to drop reserved_ptr

unsafe impl<T: Send> Send for Sender<T> {}
