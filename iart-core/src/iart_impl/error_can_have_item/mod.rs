#![doc = include_str!("../../../doc/modules/error_can_have_item.md")]

#[cfg(feature = "for-nightly-allocator-api-support")]
mod alloc_api;
#[cfg(not(feature = "for-nightly-allocator-api-support"))]
mod non_alloc_api;
