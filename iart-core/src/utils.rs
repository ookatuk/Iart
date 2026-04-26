#![doc = include_str!("../doc/modules/utils.md")]

#[cfg(not(feature = "for-nightly-likely-optimization"))]
#[inline(always)]
#[doc = include_str!("../doc/fn/likely-opt-place_folder.md")]
pub const fn unlikely(x: bool) -> bool {
    x
}

#[cfg(not(feature = "for-nightly-likely-optimization"))]
#[inline(always)]
#[cold]
#[doc = include_str!("../doc/fn/likely-opt-place_folder.md")]
pub const fn cold_path() {}

#[cfg(feature = "for-nightly-likely-optimization")]
pub use core::hint::{cold_path, unlikely};

#[allow(unused)]
#[doc = include_str!("../doc/fn/const_str_to_usize.md")]
pub const fn const_str_to_usize(s: &str) -> usize {
    let mut res = 0;
    let b = s.as_bytes();
    let mut i = 0;
    while i < b.len() {
        if b[i] < b'0' || b[i] > b'9' {
            panic!("Invalid character in environment variable! Only digits '0'-'9' are allowed.");
        }

        res = res * 10 + (b[i] - b'0') as usize;
        i += 1;
    }
    res
}

#[cfg(not(feature = "no-alloc"))]
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
                as unsafe fn(
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
                as unsafe fn(
                    Box<dyn core::any::Any + Send + Sync + 'static>,
                ) -> Box<dyn crate::types::IartErr + Send + Sync + 'static>,
        )
    };

    ($err_type:ty, $alloc:ty) => {{
        let to_fn: unsafe fn(
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

        let from_fn: unsafe fn(
            Box<dyn core::any::Any + Send + Sync + 'static, $alloc>,
        ) -> Box<
            dyn crate::types::IartErr<$alloc> + Send + Sync + 'static,
            $alloc,
        > = |any| {
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

#[cfg(feature = "no-alloc")]
macro_rules! jen_fns {
    ($err_type:ty) => {
        (
            (|err: &'static (dyn crate::types::IartErr + Send + Sync + 'static)| {
                let concrete_ptr = err as *const (dyn crate::types::IartErr + Send + Sync + 'static)
                    as *const (dyn crate::types::IartErr + Send + Sync + 'static)
                    as *const $err_type;
                unsafe { &(*concrete_ptr) as &'static (dyn core::any::Any + Send + Sync + 'static) }
            })
                as unsafe fn(
                    &'static (dyn crate::types::IartErr + Send + Sync + 'static),
                ) -> &'static (dyn core::any::Any + Send + Sync + 'static),
            (|any: &'static (dyn core::any::Any + Send + Sync + 'static)| {
                let concrete_ptr = any as *const (dyn core::any::Any + Send + Sync + 'static)
                    as *const (dyn core::any::Any + Send + Sync + 'static)
                    as *const $err_type;
                unsafe {
                    &(*concrete_ptr) as &'static (dyn crate::types::IartErr + Send + Sync + 'static)
                }
            })
                as unsafe fn(
                    &'static (dyn core::any::Any + Send + Sync + 'static),
                )
                    -> &'static (dyn crate::types::IartErr + Send + Sync + 'static),
        )
    };
}

#[allow(unused)]
#[doc = include_str!("../doc/fn/str_eq.md")]
pub const fn str_eq(a: &str, b: &str) -> bool {
    let a = a.as_bytes();
    let b = b.as_bytes();
    if a.len() != b.len() {
        return false;
    }
    let mut i = 0;
    while i < a.len() {
        if a[i] != b[i] {
            return false;
        }
        i += 1;
    }
    true
}
