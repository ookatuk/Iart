#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(
    feature = "for-nightly-try-support",
    feature(try_trait_v2),
    feature(try_trait_v2_residual)
)]
#![cfg_attr(feature = "for-nightly-likely-optimization", feature(likely_unlikely))]
#![cfg_attr(feature = "for-nightly-allocator-api-support", feature(allocator_api))]
#![cfg_attr(
    feature = "for-nightly-error-generic-member-access",
    feature(error_generic_member_access)
)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

extern crate alloc;
extern crate core;

#[macro_use]
mod utils;

pub mod events;
mod iart_impl;
#[cfg(test)]
mod tests;
mod types;

pub use types::*;

#[cfg(all(feature = "core_error-support", feature = "std"))]
compile_error!(
    "Feature 'core_error-support' cannot be used with 'std'. \
     Please disable 'core_error-support' when building for std targets."
);

use crate::utils::const_str_to_usize;
use core::ptr;
use core::sync::atomic::{AtomicPtr, Ordering};

#[allow(unused)]
pub const BACK_TRACE_MAX: usize = {
    if let Some(val) = option_env!("IART_TRACE_MAX") {
        const_str_to_usize(val)
    } else {
        1024
    }
};

/// last: A system where new data overwrites old data.
/// first: When a new one arrives, if the limit is reached, it will skip over the old one instead of overwriting it.
/// good: A system that caused the error is not deleted; instead, the old version that was used as an intermediary is deleted and a new version is installed.
#[allow(unused)]
pub const TRACE_REMOVE_TYPE: &str = {
    if let Some(s) = option_env!("IART_TRACE_TYPE") {
        s
    } else {
        "good"
    }
};

/// If the data is consecutive and originates from the same location, do not record it.
#[allow(unused)]
pub const TRACE_UNIQUE: bool = !cfg!(feature = "no-trace-dedup");

static HANDLER: AtomicPtr<()> = AtomicPtr::new(ptr::null_mut());

#[inline]
pub fn set_handler(f: IartLogger) {
    HANDLER.store(f as *mut (), Ordering::SeqCst);
}

#[inline]
pub fn is_initialized_handler() -> bool {
    !HANDLER.load(Ordering::Acquire).is_null()
}
