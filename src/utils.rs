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

macro_rules! jen_fns {
    ($err_type:ty) => {
        (
            (|err: Box<dyn crate::types::IartErr + Send + Sync + 'static>| {
                let raw_ptr = Box::into_raw(err);
                let concrete_ptr = raw_ptr
                    as *mut (dyn crate::types::IartErr + Send + Sync + 'static)
                    as *mut $err_type;
                unsafe {
                    let b = Box::from_raw(concrete_ptr);
                    b as Box<dyn core::any::Any + Send + Sync + 'static>
                }
            })
                as fn(
                    Box<dyn crate::types::IartErr + Send + Sync + 'static>,
                ) -> Box<dyn core::any::Any + Send + Sync + 'static>,
            (|any: Box<dyn core::any::Any + Send + Sync + 'static>| {
                let raw_ptr = Box::into_raw(any);
                let concrete_ptr =
                    raw_ptr as *mut (dyn core::any::Any + Send + Sync + 'static) as *mut $err_type;
                unsafe {
                    let b = Box::from_raw(concrete_ptr);
                    b as Box<dyn crate::types::IartErr + Send + Sync + 'static>
                }
            })
                as fn(
                    Box<dyn core::any::Any + Send + Sync + 'static>,
                ) -> Box<dyn crate::types::IartErr + Send + Sync + 'static>,
        )
    };

    ($err_type:ty, $alloc:ty) => {{
        let to_fn: fn(
            Box<dyn crate::types::IartErr<$alloc> + Send + Sync + 'static, $alloc>,
        ) -> Box<dyn core::any::Any + Send + Sync + 'static, $alloc> = |err| {
            let alloc = Box::allocator(&err).clone();
            let raw_ptr =
                Box::leak(err) as *mut (dyn crate::types::IartErr<$alloc> + Send + Sync + 'static);
            let concrete_ptr = raw_ptr as *mut $err_type;
            unsafe {
                let b = Box::from_raw_in(concrete_ptr, alloc);
                b as Box<dyn core::any::Any + Send + Sync + 'static, $alloc>
            }
        };

        let from_fn: fn(
            Box<dyn core::any::Any + Send + Sync + 'static, $alloc>,
        )
            -> Box<dyn crate::types::IartErr<$alloc> + Send + Sync + 'static, $alloc> = |any| {
            let alloc = Box::allocator(&any).clone();
            let raw_ptr = Box::leak(any) as *mut (dyn core::any::Any + Send + Sync + 'static);
            let concrete_ptr = raw_ptr as *mut $err_type;
            unsafe {
                let b = Box::from_raw_in(concrete_ptr, alloc);
                b as Box<dyn crate::types::IartErr<$alloc> + Send + Sync + 'static, $alloc>
            }
        };

        (to_fn, from_fn)
    }};
}
