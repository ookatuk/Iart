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

pub struct ToResultRet<T: 'static, Item = ()> {
    #[cfg(feature = "alloc")]
    pub error_data: Result<(), (Box<T>, ErrorDetail)>,
    #[cfg(not(feature = "alloc"))]
    pub error_data: Result<(), (&'static T, ErrorDetail)>,

    #[cfg(all(feature = "alloc", feature = "allow-backtrace-logging"))]
    pub backtrace: Option<VecDeque<&'static Location<'static>>>,
    #[cfg(all(not(feature = "alloc"), feature = "allow-backtrace-logging"))]
    pub backtrace: Option<[Option<&'static Location<'static>>; BACK_TRACE_MAX]>,

    pub item: Option<Item>,
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

    #[doc = include_str!("../doc/structs/Trans.md")]
    #[derive(Clone, Copy, Debug)]
    pub struct Trans {
        #[cfg(feature = "alloc")]
        pub to_any:
            unsafe fn(Box<dyn IartErr + Send + Sync>) -> Box<dyn core::any::Any + Send + Sync>,
        #[cfg(feature = "alloc")]
        pub from_any:
            unsafe fn(Box<dyn core::any::Any + Send + Sync>) -> Box<dyn IartErr + Send + Sync>,

        #[cfg(not(feature = "alloc"))]
        pub to_any: unsafe fn(
            &'static (dyn IartErr + Send + Sync),
        ) -> &'static (dyn core::any::Any + Send + Sync),
        #[cfg(not(feature = "alloc"))]
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
        #[doc = include_str!("../doc/variable/IartHandleDetails/detail.md")]
        pub detail: Option<&'a ErrorDetail>,

        pub is_err: Option<bool>,

        #[doc = include_str!("../doc/variable/IartHandleDetails/log.md")]
        #[cfg(all(feature = "allow-backtrace-logging", feature = "alloc"))]
        pub log: Option<&'a VecDeque<&'static Location<'static>>>,
        #[doc = include_str!("../doc/variable/IartHandleDetails/log.md")]
        #[cfg(all(feature = "allow-backtrace-logging", not(feature = "alloc")))]
        pub log: Option<&'a [Option<&'static Location<'static>>]>,
    }

    #[must_use]
    #[doc = include_str!("../doc/structs/Iart.md")]
    pub struct Iart<Item = ()> {
        #[doc = include_str!("../doc/variable/Iart/handled.md")]
        pub(crate) handled: bool,

        #[doc = include_str!("../doc/variable/Iart/data.md")]
        pub(crate) data: Option<Result<(), ErrorDetail>>,

        #[doc = include_str!("../doc/variable/global/item.md")]
        pub(crate) item: Option<Item>,

        #[cfg(all(feature = "allow-backtrace-logging", feature = "alloc"))]
        #[doc = include_str!("../doc/variable/Iart/log.md")]
        pub(crate) log: Option<VecDeque<&'static Location<'static>>>,

        #[cfg(all(feature = "allow-backtrace-logging", not(feature = "alloc")))]
        #[doc = include_str!("../doc/variable/Iart/log.md")]
        pub(crate) log: Option<[Option<&'static Location<'static>>; BACK_TRACE_MAX]>,

        #[doc = include_str!("../doc/variable/global/trans_fns.md")]
        pub(crate) trans_fns: Option<Trans>,
    }
}

#[cfg(feature = "for-nightly-allocator-api-support")]
mod api_impl {
    use crate::events::IartEvent;
    use crate::types::ErrorDetail;
    use alloc::boxed::Box;
    #[cfg(feature = "allow-backtrace-logging")]
    use alloc::collections::VecDeque;
    use core::alloc::Allocator;
    use core::fmt::{Debug, Display};
    #[cfg(feature = "allow-backtrace-logging")]
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
        pub to_any: unsafe fn(
            Box<dyn IartErr<A> + Send + Sync, A>,
        ) -> Box<dyn core::any::Any + Send + Sync, A>,
        pub from_any: unsafe fn(
            Box<dyn core::any::Any + Send + Sync, A>,
        ) -> Box<dyn IartErr<A> + Send + Sync, A>,
    }

    pub type IartLogger<A = alloc::alloc::Global> =
        for<'a, 'b> fn(event: IartEvent<'a, 'b>, iart: IartHandleDetails<A>) -> core::fmt::Result;

    #[allow(unused)]
    #[doc = include_str!("../doc/structs/IartHandleDetails.md")]
    #[derive(Clone, Debug)]
    pub struct IartHandleDetails<'a, A: alloc::alloc::Allocator + Clone = alloc::alloc::Global> {
        #[doc = include_str!("../doc/variable/IartHandleDetails/detail.md")]
        pub detail: Option<&'a ErrorDetail<A>>,

        #[cfg(feature = "allow-backtrace-logging")]
        #[doc = include_str!("../doc/variable/IartHandleDetails/log.md")]
        pub log: Option<&'a VecDeque<&'static Location<'static>, A>>,

        pub is_err: Option<bool>,
    }

    #[must_use]
    #[doc = include_str!("../doc/structs/Iart.md")]
    pub struct Iart<Item = (), A: alloc::alloc::Allocator + Clone + 'static = alloc::alloc::Global> {
        #[doc = include_str!("../doc/variable/Iart/handled.md")]
        pub(crate) handled: bool,

        #[doc = include_str!("../doc/variable/Iart/data.md")]
        pub(crate) data: Option<Result<(), ErrorDetail<A>>>,

        #[cfg(feature = "allow-backtrace-logging")]
        #[doc = include_str!("../doc/variable/Iart/log.md")]
        pub(crate) log: Option<VecDeque<&'static Location<'static>, A>>,

        #[doc = include_str!("../doc/variable/global/item.md")]
        pub(crate) item: Option<Item>,

        #[doc = include_str!("../doc/variable/Iart/allocator.md")]
        pub(crate) allocator: A,

        #[doc = include_str!("../doc/variable/global/trans_fns.md")]
        pub(crate) trans_fns: Option<Trans<A>>,
    }
}

#[cfg(feature = "for-nightly-allocator-api-support")]
#[doc(inline)]
pub use api_impl::*;
#[cfg(not(feature = "for-nightly-allocator-api-support"))]
#[doc(inline)]
pub use non_api_impl::*;

#[cfg(all(not(feature = "alloc"), feature = "allow-backtrace-logging"))]
use crate::BACK_TRACE_MAX;
#[cfg(feature = "alloc")]
use alloc::borrow::Cow;
#[cfg(feature = "alloc")]
use alloc::boxed::Box;
#[cfg(all(feature = "alloc", feature = "allow-backtrace-logging"))]
use alloc::collections::VecDeque;
use core::fmt::{Display, Formatter};
#[cfg(feature = "allow-backtrace-logging")]
use core::panic::Location;

#[derive(Debug, Clone)]
#[doc = include_str!("../doc/structs/DummyErr.md")]
pub struct DummyErr {}

impl core::error::Error for DummyErr {}

impl Display for DummyErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "Dummy Err")
    }
}
