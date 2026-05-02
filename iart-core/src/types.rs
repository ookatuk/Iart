#![doc = include_str!("../doc/modules/types.md")]

#[must_use]
#[derive(Debug)]
#[doc = include_str!("../doc/structs/ErrorDetail.md")]
#[cfg(feature = "for-nightly-allocator-api-support")]
pub struct ErrorDetail<A: alloc::alloc::Allocator + Clone = alloc::alloc::Global> {
    #[cfg(feature = "alloc")]
    #[doc = include_str!("../doc/variable/ErrorDetail/ty.md")]
    pub ty: Option<Box<dyn IartErr<A> + Send + Sync, A>>,
    #[cfg(not(feature = "alloc"))]
    #[doc = include_str!("../doc/variable/ErrorDetail/ty.md")]
    pub ty: Option<&'static (dyn IartErr<A> + Send + Sync, A)>,

    #[doc = include_str!("../doc/variable/global/trans_fns.md")]
    pub(crate) trans_fns: Trans<A>,

    #[cfg(feature = "alloc")]
    #[doc = include_str!("../doc/variable/ErrorDetail/desc.md")]
    pub desc: Option<Cow<'static, str>>,
    #[cfg(not(feature = "alloc"))]
    #[doc = include_str!("../doc/variable/ErrorDetail/desc.md")]
    pub desc: Option<&'static str>,
}

#[doc = include_str!("../doc/structs/ToResultRet.md")]
pub struct ToResultRet<T: 'static, Item = ()> {
    #[cfg(feature = "alloc")]
    #[doc = include_str!("../doc/variable/global/data.md")]
    pub error_data: Result<(), (Box<T>, ErrorDetail)>,
    #[cfg(not(feature = "alloc"))]
    #[doc = include_str!("../doc/variable/global/data.md")]
    pub error_data: Result<(), (&'static T, ErrorDetail)>,

    #[cfg(feature = "allow-backtrace-logging")]
    #[doc = include_str!("../doc/variable/global/log.md")]
    pub backtrace: Option<IartLog>,

    #[doc = include_str!("../doc/variable/global/item.md")]
    pub item: Option<Item>,
}

#[doc = include_str!("../doc/structs/DownCasted.md")]
pub struct DownCasted<T: 'static> {
    #[doc = include_str!("../doc/variable/global/detail.md")]
    pub detail: ErrorDetail,
    #[cfg(feature = "alloc")]
    #[doc = include_str!("../doc/variable/DownCasted/downcast.md")]
    pub downcast: T,
    #[cfg(not(feature = "alloc"))]
    #[doc = include_str!("../doc/variable/DownCasted/downcast.md")]
    pub downcast: &'static T,
}

#[must_use]
#[derive(Debug)]
#[doc = include_str!("../doc/structs/ErrorDetail.md")]
#[cfg(not(feature = "for-nightly-allocator-api-support"))]
pub struct ErrorDetail {
    #[cfg(feature = "alloc")]
    #[doc = include_str!("../doc/variable/ErrorDetail/ty.md")]
    pub ty: Option<Box<dyn IartErr + Send + Sync>>,
    #[cfg(not(feature = "alloc"))]
    #[doc = include_str!("../doc/variable/ErrorDetail/ty.md")]
    pub ty: Option<&'static (dyn IartErr + Send + Sync)>,

    #[doc = include_str!("../doc/variable/global/trans_fns.md")]
    pub(crate) trans_fns: Trans,

    #[cfg(feature = "alloc")]
    #[doc = include_str!("../doc/variable/ErrorDetail/desc.md")]
    pub desc: Option<Cow<'static, str>>,
    #[cfg(not(feature = "alloc"))]
    #[doc = include_str!("../doc/variable/ErrorDetail/desc.md")]
    pub desc: Option<&'static str>,
}

#[cfg(not(feature = "for-nightly-allocator-api-support"))]
mod non_api_impl {
    #[cfg(all(not(feature = "alloc"), feature = "allow-backtrace-logging"))]
    use crate::BACK_TRACE_MAX;
    use crate::events::IartEvent;
    use crate::types::ErrorDetail;
    #[cfg(feature = "alloc")]
    use alloc::boxed::Box;
    #[cfg(all(feature = "allow-backtrace-logging", feature = "alloc"))]
    use alloc::collections::VecDeque;
    use core::fmt::{Debug, Display};
    #[cfg(feature = "allow-backtrace-logging")]
    use core::panic::Location;

    pub type IartLogger =
        for<'a, 'b> fn(event: IartEvent<'a, 'b>, iart: IartHandleDetails) -> core::fmt::Result;

    #[cfg(all(
        feature = "allow-backtrace-logging",
        feature = "alloc",
        not(feature = "enable-limit-trace-application-level-size")
    ))]
    pub type IartLog = VecDeque<&'static Location<'static>>; // TODO: DOC
    #[cfg(all(
        not(feature = "alloc"),
        feature = "allow-backtrace-logging",
        not(feature = "enable-limit-trace-application-level-size")
    ))]
    pub type IartLog = [Option<&'static Location<'static>>; BACK_TRACE_MAX]; // TODO: DOC

    #[cfg(all(
        feature = "allow-backtrace-logging",
        feature = "alloc",
        feature = "enable-limit-trace-application-level-size"
    ))]
    pub type IartLog = spin::MutexGuard<'static, VecDeque<&'static Location<'static>>>; // TODO: DOC

    #[cfg(all(
        not(feature = "alloc"),
        feature = "allow-backtrace-logging",
        feature = "enable-limit-trace-application-level-size"
    ))]
    pub type IartLog =
        spin::MutexGuard<'static, [Option<&'static Location<'static>>; BACK_TRACE_MAX]>; // TODO: DOC

    #[cfg(all(feature = "allow-backtrace-logging", feature = "alloc"))]
    pub type IartLogRef<'a> = &'a VecDeque<&'static Location<'static>>; // TODO: DOC
    #[cfg(all(not(feature = "alloc"), feature = "allow-backtrace-logging"))]
    pub type IartLogRef<'a> = &'a [Option<&'static Location<'static>>]; // TODO: DOC

    #[doc = include_str!("../doc/structs/Trans.md")]
    #[derive(Clone, Copy, Debug)]
    pub struct Trans {
        #[cfg(feature = "alloc")]
        #[doc = include_str!("../doc/variable/Trans/to_any.md")]
        pub to_any:
            unsafe fn(Box<dyn IartErr + Send + Sync>) -> Box<dyn core::any::Any + Send + Sync>,
        #[cfg(feature = "alloc")]
        #[doc = include_str!("../doc/variable/Trans/from_any.md")]
        pub from_any:
            unsafe fn(Box<dyn core::any::Any + Send + Sync>) -> Box<dyn IartErr + Send + Sync>,

        #[cfg(not(feature = "alloc"))]
        #[doc = include_str!("../doc/variable/Trans/to_any.md")]
        pub to_any: unsafe fn(
            &'static (dyn IartErr + Send + Sync),
        ) -> &'static (dyn core::any::Any + Send + Sync),
        #[cfg(not(feature = "alloc"))]
        #[doc = include_str!("../doc/variable/Trans/from_any.md")]
        pub from_any: unsafe fn(
            &'static (dyn core::any::Any + Send + Sync),
        ) -> &'static (dyn IartErr + Send + Sync),
    }

    #[cfg(not(feature = "for-nightly-error-generic-member-access"))]
    #[doc = include_str!("../doc/trait/IartErr.md")]
    #[must_use]
    pub trait IartErr: Debug + Display + Send + Sync {
        #[cfg(feature = "alloc")]
        fn clone_box(&self) -> Box<dyn IartErr + Send + Sync>;
    }

    #[cfg(feature = "for-nightly-error-generic-member-access")]
    #[must_use]
    #[doc = include_str!("../doc/trait/IartErr.md")]
    pub trait IartErr: Debug + Display + core::error::Error + Send + Sync {
        #[cfg(feature = "alloc")]
        fn clone_box(&self) -> Box<dyn IartErr + Send + Sync>;
    }

    #[allow(unused)]
    #[derive(Debug, Clone)]
    #[doc = include_str!("../doc/structs/IartHandleDetails.md")]
    pub struct IartHandleDetails<'a> {
        #[doc = include_str!("../doc/variable/global/detail.md")]
        pub detail: Option<&'a ErrorDetail>,

        pub is_err: Option<bool>,

        #[doc = include_str!("../doc/variable/global/log.md")]
        #[cfg(all(feature = "allow-backtrace-logging", feature = "alloc"))]
        pub log: Option<&'a IartLog>,
        #[doc = include_str!("../doc/variable/global/log.md")]
        #[cfg(all(feature = "allow-backtrace-logging", not(feature = "alloc")))]
        pub log: Option<IartLogRef<'a>>,
    }

    #[must_use]
    #[doc = include_str!("../doc/structs/Iart.md")]
    pub struct Iart<Item = ()> {
        #[doc = include_str!("../doc/variable/Iart/handled.md")]
        pub(crate) handled: bool,

        #[doc = include_str!("../doc/variable/global/data.md")]
        pub(crate) data: Option<Result<(), ErrorDetail>>,

        #[doc = include_str!("../doc/variable/global/item.md")]
        pub(crate) item: Option<Item>,

        #[cfg(all(feature = "allow-backtrace-logging", feature = "alloc"))]
        #[doc = include_str!("../doc/variable/global/log.md")]
        pub(crate) log: Option<IartLog>,

        #[cfg(all(feature = "allow-backtrace-logging", not(feature = "alloc")))]
        #[doc = include_str!("../doc/variable/global/log.md")]
        pub(crate) log: Option<IartLog>,

        #[doc = include_str!("../doc/variable/global/trans_fns.md")]
        pub(crate) trans_fns: Option<Trans>,

        #[cfg(feature = "enable-pending-tracker")]
        #[doc = include_str!("../doc/variable/Iart/tracking_id.md")]
        pub(crate) tracking_id: Option<usize>,
    }

    #[doc = include_str!("../doc/structs/GetErrRet.md")]
    pub struct GetErrRet<Item> {
        #[doc = include_str!("../doc/variable/global/detail.md")]
        pub detail: ErrorDetail,
        #[doc = include_str!("../doc/variable/global/item.md")]
        pub item: Option<Item>,
    }
}

#[cfg(feature = "for-nightly-allocator-api-support")]
mod api_impl {
    use crate::events::IartEvent;
    use crate::types::ErrorDetail;
    use alloc::boxed::Box;
    use alloc::collections::VecDeque;
    use core::alloc::Allocator;
    use core::fmt::{Debug, Display};
    use core::panic::Location;

    #[cfg(feature = "for-nightly-error-generic-member-access")]
    #[must_use]
    #[doc = include_str!("../doc/trait/IartErr.md")]
    pub trait IartErr<A: Allocator + Clone = alloc::alloc::Global>:
        Debug + Display + core::error::Error
    where
        Self: 'static,
    {
        #[cfg(feature = "alloc")]
        fn clone_box_in<'a>(&self, alloc: A) -> Box<dyn IartErr<A> + 'a + Send + Sync, A>
        where
            Self: 'a;
    }

    #[cfg(not(feature = "for-nightly-error-generic-member-access"))]
    #[must_use]
    #[doc = include_str!("../doc/trait/IartErr.md")]
    pub trait IartErr<A: Allocator + Clone = alloc::alloc::Global>: Debug + Display {
        #[cfg(feature = "alloc")]
        fn clone_box_in<'a>(&self, alloc: A) -> Box<dyn IartErr<A> + 'a + Send + Sync, A>
        where
            Self: 'a;
    }

    #[doc = include_str!("../doc/structs/Trans.md")]
    #[derive(Debug, Clone, Copy)]
    pub struct Trans<A: alloc::alloc::Allocator + Clone = alloc::alloc::Global> {
        #[doc = include_str!("../doc/variable/Trans/to_any.md")]
        pub to_any: unsafe fn(
            Box<dyn IartErr<A> + Send + Sync, A>,
        ) -> Box<dyn core::any::Any + Send + Sync, A>,
        #[doc = include_str!("../doc/variable/Trans/from_any.md")]
        pub from_any: unsafe fn(
            Box<dyn core::any::Any + Send + Sync, A>,
        ) -> Box<dyn IartErr<A> + Send + Sync, A>,
    }

    pub type IartLogger<A = alloc::alloc::Global> =
        for<'a, 'b> fn(event: IartEvent<'a, 'b>, iart: IartHandleDetails<A>) -> core::fmt::Result;

    pub type IartLog<A = alloc::alloc::Global> = VecDeque<&'static Location<'static>, A>;

    #[allow(unused)]
    #[doc = include_str!("../doc/structs/IartHandleDetails.md")]
    #[derive(Clone, Debug)]
    pub struct IartHandleDetails<'a, A: alloc::alloc::Allocator + Clone = alloc::alloc::Global> {
        #[doc = include_str!("../doc/variable/global/detail.md")]
        pub detail: Option<&'a ErrorDetail<A>>,

        #[cfg(feature = "allow-backtrace-logging")]
        #[doc = include_str!("../doc/variable/global/log.md")]
        pub log: Option<&'a IartLog<A>>,

        pub is_err: Option<bool>,
    }

    #[must_use]
    #[doc = include_str!("../doc/structs/Iart.md")]
    pub struct Iart<Item = (), A: alloc::alloc::Allocator + Clone + 'static = alloc::alloc::Global> {
        #[doc = include_str!("../doc/variable/Iart/handled.md")]
        pub(crate) handled: bool,

        #[doc = include_str!("../doc/variable/global/data.md")]
        pub(crate) data: Option<Result<(), ErrorDetail<A>>>,

        #[cfg(feature = "allow-backtrace-logging")]
        #[doc = include_str!("../doc/variable/global/log.md")]
        pub(crate) log: Option<IartLog<A>>,

        #[doc = include_str!("../doc/variable/global/item.md")]
        pub(crate) item: Option<Item>,

        #[doc = include_str!("../doc/variable/Iart/allocator.md")]
        pub(crate) allocator: A,

        #[doc = include_str!("../doc/variable/global/trans_fns.md")]
        pub(crate) trans_fns: Option<Trans<A>>,

        #[cfg(feature = "enable-pending-tracker")]
        #[doc = include_str!("../doc/variable/Iart/tracking_id.md")]
        pub(crate) tracking_id: Option<usize>,
    }

    #[doc = include_str!("../doc/structs/GetErrRet.md")]
    pub struct GetErrRet<Item, A: alloc::alloc::Allocator + Clone = alloc::alloc::Global> {
        #[doc = include_str!("../doc/variable/global/detail.md")]
        pub detail: ErrorDetail<A>,
        #[doc = include_str!("../doc/variable/global/item.md")]
        pub item: Option<Item>,
    }
}

#[cfg(feature = "for-nightly-allocator-api-support")]
#[doc(inline)]
pub use api_impl::*;
#[cfg(not(feature = "for-nightly-allocator-api-support"))]
#[doc(inline)]
pub use non_api_impl::*;

#[cfg(feature = "alloc")]
use alloc::borrow::Cow;
#[cfg(feature = "alloc")]
use alloc::boxed::Box;
use core::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
#[doc = include_str!("../doc/structs/DummyErr.md")]
pub struct DummyErr {}

impl core::error::Error for DummyErr {}

impl Display for DummyErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "Dummy Err")
    }
}
