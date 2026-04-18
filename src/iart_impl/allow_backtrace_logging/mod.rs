mod impls;
#[allow(unused_imports)]
pub use impls::*;

#[cfg(not(feature = "for-nightly-allocator-api-support"))]
mod non_alloc_api;
#[cfg(not(feature = "for-nightly-allocator-api-support"))]
#[allow(unused_imports)]
pub use non_alloc_api::*;

#[cfg(feature = "for-nightly-allocator-api-support")]
mod alloc_api;
#[cfg(feature = "for-nightly-allocator-api-support")]
#[allow(unused_imports)]
pub use alloc_api::*;
