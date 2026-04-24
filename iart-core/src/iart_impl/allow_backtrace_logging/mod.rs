#![doc = include_str!("../../../doc/modules/allow_backtrace_logging.md")]

mod impls;

#[cfg(feature = "for-nightly-allocator-api-support")]
mod alloc_api;
#[cfg(not(feature = "for-nightly-allocator-api-support"))]
mod non_alloc_api;
