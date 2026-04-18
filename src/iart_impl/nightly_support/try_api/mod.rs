#[cfg(not(feature = "for-nightly-allocator-api-support"))]
mod non_alloc_api;

#[cfg(feature = "for-nightly-allocator-api-support")]
mod alloc_api;
