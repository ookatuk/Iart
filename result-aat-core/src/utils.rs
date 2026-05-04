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

#[cfg(feature = "enable-pending-tracker")]
use crate::{RESULT_TRACK_MAX, TRACKER, TRACKER_MAX_OFFSET};
#[cfg(feature = "alloc")]
#[allow(unused_imports)]
use alloc::collections::VecDeque;

#[allow(unused_imports)]
use core::panic::Location;
#[cfg(any(
    feature = "enable-pending-tracker",
    feature = "enable-limit-trace-application-level-size"
))]
use core::sync::atomic::{AtomicUsize, Ordering};

#[cfg(feature = "allow-backtrace-logging")]
use crate::IartBacktrace;

#[allow(unused)]
use crate::BACK_TRACE_MAX;

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

#[cfg(feature = "alloc")]
macro_rules! jen_fns {
    ($err_type:ty) => {{
        let to_fn: unsafe fn(
            Box<dyn crate::types::IartErr + Send + Sync + 'static>,
        ) -> Box<dyn core::any::Any + Send + Sync + 'static> = |err: Box<
            dyn crate::types::IartErr + Send + Sync + 'static,
        >| {
            let raw_ptr = Box::into_raw(err);
            let concrete_ptr = raw_ptr as *mut (dyn crate::types::IartErr + Send + Sync + 'static)
                as *mut $err_type;
            unsafe {
                let b = Box::from_raw(concrete_ptr);
                b as Box<dyn core::any::Any + Send + Sync + 'static>
            }
        };

        let from_fn: unsafe fn(
            Box<dyn core::any::Any + Send + Sync + 'static>,
        ) -> Box<dyn crate::types::IartErr + Send + Sync + 'static> =
            |any: Box<dyn core::any::Any + Send + Sync + 'static>| {
                let raw_ptr = Box::into_raw(any);
                let concrete_ptr =
                    raw_ptr as *mut (dyn core::any::Any + Send + Sync + 'static) as *mut $err_type;
                unsafe {
                    let b = Box::from_raw(concrete_ptr);
                    b as Box<dyn crate::types::IartErr + Send + Sync + 'static>
                }
            };

        Trans {
            from_any: from_fn,
            to_any: to_fn,
        }
    }};

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

        Trans {
            from_any: from_fn,
            to_any: to_fn,
        }
    }};
}

#[cfg(not(feature = "alloc"))]
macro_rules! jen_fns {
    ($err_type:ty) => {{
        let to_fn: unsafe fn(
            &'static (dyn crate::types::IartErr + Send + Sync + 'static),
        ) -> &'static (dyn core::any::Any + Send + Sync + 'static) =
            |err: &'static (dyn crate::types::IartErr + Send + Sync + 'static)| {
                let concrete_ptr = err as *const (dyn crate::types::IartErr + Send + Sync + 'static)
                    as *const (dyn crate::types::IartErr + Send + Sync + 'static)
                    as *const $err_type;
                unsafe { &(*concrete_ptr) as &'static (dyn core::any::Any + Send + Sync + 'static) }
            };
        let from_fn: unsafe fn(
            &'static (dyn core::any::Any + Send + Sync + 'static),
        )
            -> &'static (dyn crate::types::IartErr + Send + Sync + 'static) =
            |any: &'static (dyn core::any::Any + Send + Sync + 'static)| {
                let concrete_ptr = any as *const (dyn core::any::Any + Send + Sync + 'static)
                    as *const (dyn core::any::Any + Send + Sync + 'static)
                    as *const $err_type;
                unsafe {
                    &(*concrete_ptr) as &'static (dyn crate::types::IartErr + Send + Sync + 'static)
                }
            };

        Trans {
            from_any: from_fn,
            to_any: to_fn,
        }
    }};
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

#[cfg(all(feature = "enable-pending-tracker"))]
#[doc = include_str!("../doc/fn/add_to_tracker.md")]
pub fn add_to_tracker(data: &'static Location<'static>) -> Option<usize> {
    static OFFSET: AtomicUsize = AtomicUsize::new(0);

    const ODD_TARGET: usize = if TRACKER_MAX_OFFSET > RESULT_TRACK_MAX {
        RESULT_TRACK_MAX
    } else {
        TRACKER_MAX_OFFSET
    };

    #[cfg(feature = "enable-pending-tracker-tracking-count")]
    if crate::TRACKING_COUNT.load(Ordering::Relaxed) >= RESULT_TRACK_MAX {
        return None;
    }

    let mut target = OFFSET.fetch_add(1, Ordering::Relaxed) % ODD_TARGET;

    for _ in 0..ODD_TARGET {
        let res = TRACKER
            .iter()
            .enumerate()
            .skip(target)
            .step_by(ODD_TARGET)
            .find_map(|(index, mutex)| {
                let mut lock = mutex.try_lock()?;
                if lock.is_some() {
                    return None;
                }

                // If we put the logic inside the if let block, the data might be modified before we can re-lock the mutex.
                // That's why we complete the update while the lock is still held inside the iterator.
                // (Based on my original notes, translated to English by AI.)
                *lock = Some([data; 2]);
                Some(index)
            });

        if let Some(index) = res {
            #[cfg(feature = "enable-pending-tracker-tracking-count")]
            crate::TRACKING_COUNT.fetch_add(1, Ordering::Relaxed);
            return Some(index);
        }

        target = (target + 1) % ODD_TARGET;
        OFFSET.fetch_add(1, Ordering::Relaxed);
    }
    cold_path();

    None
}

#[cfg(all(feature = "enable-pending-tracker"))]
#[inline]
#[doc = include_str!("../doc/fn/update_to_tracker.md")]
#[allow(unused)]
pub fn update_to_tracker(index: Option<usize>, data: &'static Location<'static>) {
    if let Some(index) = index {
        TRACKER[index].lock().unwrap()[1] = data;
    }
}

#[cfg(all(feature = "enable-pending-tracker"))]
#[inline]
#[doc = include_str!("../doc/fn/remove_from_tracker.md")]
pub fn remove_from_tracker(index: Option<usize>) {
    if let Some(index) = index {
        #[cfg(feature = "enable-pending-tracker-tracking-count")]
        crate::TRACKING_COUNT.fetch_sub(1, Ordering::Relaxed);
        *TRACKER[index].lock() = None;
    }
}

#[cfg(all(
    feature = "enable-limit-trace-application-level-size",
    feature = "alloc"
))]
pub fn get_trace_location(
) -> Option<spin::MutexGuard<'static, alloc::collections::VecDeque<&'static Location<'static>>>> {
    static OFFSET: AtomicUsize = AtomicUsize::new(0);

    const ODD_TARGET: usize = if crate::TRACE_DATABASE_MAX_OFFSET > crate::TRACE_DATABASE_SIZE {
        crate::TRACE_DATABASE_SIZE
    } else {
        crate::TRACE_DATABASE_MAX_OFFSET
    };

    #[cfg(feature = "enable-limit-trace-application-level-size-tracking-count")]
    if crate::TRACKING_COUNT.load(Ordering::Relaxed) >= crate::TRACE_DATABASE_SIZE {
        return None;
    }

    let mut target = OFFSET.fetch_add(1, Ordering::Relaxed) % ODD_TARGET;

    for _ in 0..ODD_TARGET {
        let res = crate::TRACE_DATABASE
            .iter()
            .enumerate()
            .skip(target)
            .step_by(ODD_TARGET)
            .find_map(|(_, mutex)| mutex.try_lock());

        if let Some(index) = res {
            #[cfg(feature = "enable-limit-trace-application-level-size-tracking-count")]
            crate::TRACKING_COUNT.fetch_add(1, Ordering::Relaxed);
            return Some(index);
        }

        target = (target + 1) % ODD_TARGET;
        OFFSET.fetch_add(1, Ordering::Relaxed);
    }
    cold_path();

    None
}

#[cfg(all(
    feature = "enable-limit-trace-application-level-size",
    not(feature = "alloc")
))]
pub fn get_trace_location(
) -> Option<spin::MutexGuard<'static, [Option<&'static Location<'static>>; crate::BACK_TRACE_MAX]>>
{
    static OFFSET: AtomicUsize = AtomicUsize::new(0);

    const ODD_TARGET: usize = if crate::TRACE_DATABASE_MAX_OFFSET > crate::TRACE_DATABASE_SIZE {
        crate::TRACE_DATABASE_SIZE
    } else {
        crate::TRACE_DATABASE_MAX_OFFSET
    };

    #[cfg(feature = "enable-limit-trace-application-level-size-tracking-count")]
    if crate::TRACKING_COUNT.load(Ordering::Relaxed) >= crate::TRACE_DATABASE_SIZE {
        return None;
    }

    let mut target = OFFSET.fetch_add(1, Ordering::Relaxed) % ODD_TARGET;

    for _ in 0..ODD_TARGET {
        let res = crate::TRACE_DATABASE
            .iter()
            .enumerate()
            .skip(target)
            .step_by(ODD_TARGET)
            .find_map(|(_, mutex)| mutex.try_lock());

        if let Some(index) = res {
            #[cfg(feature = "enable-limit-trace-application-level-size-tracking-count")]
            crate::TRACKING_COUNT.fetch_add(1, Ordering::Relaxed);
            return Some(index);
        }

        target = (target + 1) % ODD_TARGET;
        OFFSET.fetch_add(1, Ordering::Relaxed);
    }
    cold_path();

    None
}

#[track_caller]
#[cfg(all(
    feature = "allow-backtrace-logging",
    not(feature = "for-nightly-allocator-api-support")
))]
#[inline]
pub fn create_trace<const IS_OK: bool>() -> Option<IartBacktrace> {
    #[cfg(all(
        not(feature = "enable-limit-trace-application-level-size"),
        feature = "alloc"
    ))]
    let res = {
        #[allow(unused_mut)]
        let mut log: VecDeque<&'static Location<'static>> = VecDeque::new();
        if IS_OK || cfg!(feature = "allow-backtrace-logging-with-ok") {
            log.push_back(Location::caller());
        }
        Some(log)
    };

    #[cfg(all(
        not(feature = "enable-limit-trace-application-level-size"),
        not(feature = "alloc")
    ))]
    let res = {
        #[allow(unused_mut)]
        let mut log = [None; BACK_TRACE_MAX];
        if IS_OK || cfg!(feature = "allow-backtrace-logging-with-ok") {
            log[0] = Some(Location::caller());
        }
        Some(log)
    };

    #[cfg(all(
        feature = "enable-limit-trace-application-level-size",
        not(feature = "alloc")
    ))]
    let res = {
        let mut data = get_trace_location();

        if IS_OK || cfg!(feature = "allow-backtrace-logging-with-ok") {
            if let Some(log) = data.as_mut() {
                log[0] = Some(Location::caller());
            }
        }

        data
    };

    #[cfg(all(
        feature = "enable-limit-trace-application-level-size",
        feature = "alloc"
    ))]
    let res = {
        let mut data = get_trace_location();

        if IS_OK || cfg!(feature = "allow-backtrace-logging-with-ok") {
            if let Some(log) = data.as_mut() {
                log.push_back(Location::caller());
            }
        }

        data
    };

    res
}

#[track_caller]
#[cfg(all(
    feature = "allow-backtrace-logging",
    feature = "for-nightly-allocator-api-support"
))]
pub fn create_trace<const IS_OK: bool, A: alloc::alloc::Allocator>(
    allocator: A,
) -> Option<IartBacktrace<A>> {
    let mut log = VecDeque::new_in(allocator);
    if IS_OK || cfg!(feature = "allow-backtrace-logging-with-ok") {
        log.push_back(Location::caller());
    }
    Some(log)
}
