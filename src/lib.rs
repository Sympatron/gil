#![doc = include_str!("../README.md")]

#[allow(unused_imports)]
#[cfg(not(feature = "loom"))]
pub(crate) use std::{
    alloc, cell, hint,
    sync::{self, atomic},
    thread,
};

#[allow(unused_imports)]
#[cfg(feature = "loom")]
pub(crate) use loom::{
    alloc, cell, hint,
    sync::{self, atomic},
    thread,
};

pub mod mpsc;
mod padded;
pub mod spsc;
