#![doc = include_str!("../../../doc/modules/check_unused_result.md")]

#[cfg(feature = "for-nightly-allocator-api-support")]
mod alloc_api;
#[cfg(not(feature = "for-nightly-allocator-api-support"))]
mod non_alloc_api;
