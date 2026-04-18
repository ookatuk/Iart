#[cfg(not(feature = "for-nightly-likely-optimization"))]
#[inline(always)]
pub const fn unlikely(x: bool) -> bool {
    x
}

#[cfg(not(feature = "for-nightly-likely-optimization"))]
#[inline(always)]
pub const fn cold_path() {}

#[cfg(feature = "for-nightly-likely-optimization")]
pub use core::hint::{cold_path, unlikely};

#[allow(unused)]
pub const fn const_str_to_usize(s: &str) -> usize {
    let mut res = 0;
    let b = s.as_bytes();
    let mut i = 0;
    while i < b.len() {
        res = res * 10 + (b[i] - b'0') as usize;
        i += 1;
    }
    res
}
