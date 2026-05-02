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
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../doc/modules/lib.md")]

#[cfg(feature = "alloc")]
extern crate alloc;
extern crate core;

#[macro_use]
mod utils;

pub mod events;
mod iart_impl;
#[cfg(test)]
mod tests;
mod types;

#[cfg(all(feature = "enable-pending-tracker", feature = "alloc"))]
use alloc::vec::Vec;
#[cfg(any(
    feature = "enable-pending-tracker",
    feature = "enable-limit-trace-application-level-size"
))]
use core::panic::Location;
#[cfg(all(
    any(
        feature = "enable-pending-tracker",
        feature = "enable-limit-trace-application-level-size"
    ),
    feature = "alloc"
))]
use spin::Lazy;
pub use types::*;

#[cfg(all(feature = "core_error-support", feature = "std"))]
compile_error!(
    "Feature 'core_error-support' cannot be used with 'std'. \
     Please disable 'core_error-support' when building for std targets."
);

#[cfg(all(
    feature = "enable-limit-trace-application-level-size",
    feature = "for-nightly-allocator-api-support"
))]
compile_error!(
    "Feature 'enable-limit-trace-application-level-size' cannot be used with 'for-nightly-allocator-api-support'."
);

#[cfg(all(feature = "std", feature = "enable-default-handler"))]
use crate::events::IartEvent;

#[cfg(feature = "enable-pending-tracker-tracking-count")]
use core::sync::atomic::AtomicUsize;

use crate::utils::{const_str_to_usize, str_eq};
use core::sync::atomic::{AtomicPtr, Ordering};
use spin::Once;

#[allow(unused)]
pub const BACK_TRACE_MAX: usize = const_str_to_usize(env!("IART_TRACE_MAX"));
#[allow(unused)]
#[cfg(feature = "enable-pending-tracker")]
pub const RESULT_TRACK_MAX: usize = const_str_to_usize(env!("IART_TRACK_MAX"));

#[allow(unused)]
#[doc = include_str!("../doc/variable/TRACE_REMOVE_TYPE.md")]
pub const TRACE_REMOVE_TYPE: &str = {
    let s = env!("IART_TRACE_TYPE");
    if str_eq(s, "good") || str_eq(s, "first") || str_eq(s, "last") {
        s
    } else {
        panic!("Invalid IART_TRACE_TYPE!");
    }
};

#[cfg(feature = "enable-limit-trace-application-level-size")]
const TRACE_DATABASE_SIZE: usize = const_str_to_usize(env!("IART_TRACE_DATABASE_SIZE")); // TODO

#[cfg(all(
    feature = "enable-limit-trace-application-level-size",
    feature = "alloc"
))]
static TRACE_DATA_BASE: Lazy<
    Vec<spin::Mutex<alloc::collections::VecDeque<&'static Location<'static>>>>,
> = // TODO
    Lazy::new(|| {
        (0..TRACE_DATABASE_SIZE)
            .map(|_| spin::Mutex::new(alloc::collections::VecDeque::new()))
            .collect()
    });

#[cfg(all(
    feature = "enable-limit-trace-application-level-size",
    not(feature = "alloc")
))]
static TRACE_DATA_BASE: [spin::Mutex<[Option<&'static Location<'static>>; BACK_TRACE_MAX]>;
    TRACE_DATABASE_SIZE] =
    [const { spin::Mutex::new([None; BACK_TRACE_MAX]) }; TRACE_DATABASE_SIZE]; // TODO

#[cfg(all(feature = "enable-limit-trace-application-level-size"))]
const TRACE_DATABASE_MAX_OFFSET: usize = const_str_to_usize(env!("IART_TRACKER_MAX_OFFSET")); // TODO

#[allow(unused)]
#[doc = include_str!("../doc/variable/TRACE_UNIQUE.md")]
pub const TRACE_UNIQUE: bool = !cfg!(feature = "no-trace-dedup");

#[doc = include_str!("../doc/variable/HANDLER_CREATED.md")]
static HANDLER_CREATED: Once = Once::new();

#[doc = include_str!("../doc/variable/HANDLER.md")]
static HANDLER: AtomicPtr<()> = AtomicPtr::new(
    #[cfg(all(
        feature = "std",
        feature = "enable-default-handler",
        feature = "for-nightly-allocator-api-support"
    ))]
    {
        default_handler::<std::alloc::Global> as *mut ()
    },
    #[cfg(all(
        feature = "std",
        feature = "enable-default-handler",
        not(feature = "for-nightly-allocator-api-support")
    ))]
    {
        default_handler as *mut ()
    },
    #[cfg(not(all(feature = "std", feature = "enable-default-handler")))]
    core::ptr::null_mut(),
);

#[cfg(all(feature = "enable-pending-tracker", feature = "alloc"))]
#[doc = include_str!("../doc/variable/TRACKER.md")]
static TRACKER: Lazy<Vec<spin::Mutex<Option<[&'static Location<'static>; 2]>>>> = Lazy::new(|| {
    (0..RESULT_TRACK_MAX)
        .map(|_| spin::Mutex::new(None))
        .collect()
});

#[cfg(all(feature = "enable-pending-tracker", not(feature = "alloc")))]
#[doc = include_str!("../doc/variable/TRACKER.md")]
static TRACKER: [spin::Mutex<Option<[&'static Location<'static>; 2]>>; RESULT_TRACK_MAX] =
    [const { spin::Mutex::new(None) }; RESULT_TRACK_MAX];

#[cfg(feature = "enable-pending-tracker-tracking-count")]
#[doc = include_str!("../doc/variable/TRACKING_COUNT.md")]
static TRACKING_COUNT: AtomicUsize = AtomicUsize::new(0);

#[cfg(all(feature = "enable-pending-tracker"))]
#[doc = include_str!("../doc/variable/TRACKER_MAX_OFFSET.md")]
const TRACKER_MAX_OFFSET: usize = const_str_to_usize(env!("IART_TRACKER_MAX_OFFSET"));

#[inline]
#[doc = include_str!("../doc/fn/set_handler.md")]
pub fn set_handler(f: IartLogger) -> bool {
    if HANDLER_CREATED.is_completed() {
        return false;
    }

    HANDLER_CREATED.call_once(|| {});
    HANDLER.store(f as *mut (), Ordering::SeqCst);
    true
}

#[inline]
#[doc = include_str!("../doc/fn/is_initialized_handler.md")]
pub fn is_initialized_handler() -> bool {
    !HANDLER.load(Ordering::Acquire).is_null()
}

#[inline]
#[cfg(feature = "enable-pending-tracker")]
#[doc = include_str!("../doc/fn/get_current_tracking_data.md")]
pub fn get_current_tracking_data() -> &'static [spin::Mutex<Option<[&'static Location<'static>; 2]>>]
{
    TRACKER.as_slice()
}

#[inline]
#[cfg(feature = "enable-pending-tracker")]
#[doc = include_str!("../doc/fn/is_found_pending_data.md")]
pub fn is_found_pending_data() -> bool {
    #[cfg(not(feature = "enable-pending-tracker-tracking-count"))]
    let res = {
        for i in TRACKER.iter() {
            if i.lock().is_some() {
                return true;
            }
        }
        false
    };

    #[cfg(feature = "enable-pending-tracker-tracking-count")]
    let res = TRACKING_COUNT.load(Ordering::SeqCst) >= RESULT_TRACK_MAX;

    res
}

#[doc = include_str!("../doc/fn/default_handler.md")]
#[cfg(all(
    feature = "std",
    feature = "enable-default-handler",
    not(feature = "for-nightly-allocator-api-support")
))]
pub fn default_handler(event: IartEvent, iart: IartHandleDetails) -> std::fmt::Result {
    match event {
        IartEvent::DroppedWithoutCheck => {
            eprintln!("IART dropped without check! {:?}", iart);
        }
        _ => {}
    }

    Ok(())
}

#[doc = include_str!("../doc/fn/default_handler.md")]
#[cfg(all(
    feature = "std",
    feature = "enable-default-handler",
    feature = "for-nightly-allocator-api-support"
))]
pub fn default_handler<A: alloc::alloc::Allocator + Clone + core::fmt::Debug>(
    event: IartEvent,
    iart: IartHandleDetails<A>,
) -> std::fmt::Result {
    match event {
        IartEvent::DroppedWithoutCheck => {
            eprintln!("IART dropped without check! {:?}", iart);
        }
        _ => {}
    }

    Ok(())
}
