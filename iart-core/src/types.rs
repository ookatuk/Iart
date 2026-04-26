#![doc = include_str!("../doc/modules/types.md")]

#[must_use]
#[derive(Debug)]
#[doc = include_str!("../doc/structs/ErrorDetail.md")]
#[cfg(feature = "for-nightly-allocator-api-support")]
pub struct ErrorDetail<A: alloc::alloc::Allocator + Clone = alloc::alloc::Global> {
    #[cfg(not(feature = "no-alloc"))]
    #[doc = include_str!("../doc/variable/ErrorDetail/ty.md")]
    pub ty: Option<Box<dyn IartErr<A> + Send + Sync, A>>,
    #[cfg(feature = "no-alloc")]
    #[doc = include_str!("../doc/variable/ErrorDetail/ty.md")]
    pub ty: Option<&'static (dyn IartErr<A> + Send + Sync, A)>,

    #[cfg(not(feature = "no-alloc"))]
    #[doc = include_str!("../doc/variable/global/trans_fns.md")]
    pub(crate) trans_fns: (
        unsafe fn(Box<dyn IartErr<A> + Send + Sync, A>) -> Box<dyn core::any::Any + Send + Sync, A>,
        unsafe fn(Box<dyn core::any::Any + Send + Sync, A>) -> Box<dyn IartErr<A> + Send + Sync, A>,
    ),
    #[cfg(feature = "no-alloc")]
    #[doc = include_str!("../doc/variable/global/trans_fns.md")]
    pub(crate) trans_fns: (
        unsafe fn(
            &'static (dyn IartErr<A> + Send + Sync),
        ) -> &'static (dyn core::any::Any + Send + Sync),
        unsafe fn(
            &'static (dyn core::any::Any + Send + Sync),
        ) -> &'static (dyn IartErr<A> + Send + Sync),
    ),

    #[cfg(not(feature = "no-alloc"))]
    #[doc = include_str!("../doc/variable/ErrorDetail/desc.md")]
    pub desc: Option<Cow<'static, str>>,
    #[cfg(feature = "no-alloc")]
    #[doc = include_str!("../doc/variable/ErrorDetail/desc.md")]
    pub desc: Option<&'static str>,
}

#[must_use]
#[derive(Debug)]
#[doc = include_str!("../doc/structs/ErrorDetail.md")]
#[cfg(not(feature = "for-nightly-allocator-api-support"))]
pub struct ErrorDetail {
    #[cfg(not(feature = "no-alloc"))]
    #[doc = include_str!("../doc/variable/ErrorDetail/ty.md")]
    pub ty: Option<Box<dyn IartErr + Send + Sync>>,
    #[cfg(feature = "no-alloc")]
    #[doc = include_str!("../doc/variable/ErrorDetail/ty.md")]
    pub ty: Option<&'static (dyn IartErr + Send + Sync)>,

    #[cfg(not(feature = "no-alloc"))]
    #[doc = include_str!("../doc/variable/global/trans_fns.md")]
    pub(crate) trans_fns: (
        unsafe fn(Box<dyn IartErr + Send + Sync>) -> Box<dyn core::any::Any + Send + Sync>,
        unsafe fn(Box<dyn core::any::Any + Send + Sync>) -> Box<dyn IartErr + Send + Sync>,
    ),
    #[cfg(feature = "no-alloc")]
    #[doc = include_str!("../doc/variable/global/trans_fns.md")]
    pub(crate) trans_fns: (
        unsafe fn(
            &'static (dyn IartErr + Send + Sync),
        ) -> &'static (dyn core::any::Any + Send + Sync),
        unsafe fn(
            &'static (dyn core::any::Any + Send + Sync),
        ) -> &'static (dyn IartErr + Send + Sync),
    ),

    #[cfg(not(feature = "no-alloc"))]
    #[doc = include_str!("../doc/variable/ErrorDetail/desc.md")]
    pub desc: Option<Cow<'static, str>>,
    #[cfg(feature = "no-alloc")]
    #[doc = include_str!("../doc/variable/ErrorDetail/desc.md")]
    pub desc: Option<&'static str>,
}

#[cfg(not(feature = "for-nightly-allocator-api-support"))]
mod non_api_impl {
    #[cfg(all(feature = "no-alloc", feature = "allow-backtrace-logging"))]
    use crate::BACK_TRACE_MAX;
    use crate::events::IartEvent;
    use crate::types::ErrorDetail;
    #[cfg(not(feature = "no-alloc"))]
    use alloc::boxed::Box;
    #[cfg(all(feature = "allow-backtrace-logging", not(feature = "no-alloc")))]
    use alloc::collections::VecDeque;
    use core::fmt::{Debug, Display};
    #[cfg(feature = "allow-backtrace-logging")]
    use core::panic::Location;

    pub type IartLogger =
        for<'a, 'b> fn(event: IartEvent<'a, 'b>, iart: IartHandleDetails) -> core::fmt::Result;

    #[cfg(not(feature = "for-nightly-error-generic-member-access"))]
    #[doc = include_str!("../doc/trait/IartErr.md")]
    #[must_use]
    pub trait IartErr: Debug + Display + Send + Sync {
        #[cfg(not(feature = "no-alloc"))]
        fn clone_box(&self) -> Box<dyn IartErr + Send + Sync>;
    }

    #[cfg(feature = "for-nightly-error-generic-member-access")]
    #[must_use]
    #[doc = include_str!("../doc/trait/IartErr.md")]
    pub trait IartErr: Debug + Display + core::error::Error + Send + Sync {
        #[cfg(not(feature = "no-alloc"))]
        fn clone_box(&self) -> Box<dyn IartErr + Send + Sync>;
    }

    #[allow(unused)]
    #[derive(Debug, Clone)]
    #[doc = include_str!("../doc/structs/IartHandleDetails.md")]
    pub struct IartHandleDetails<'a> {
        #[doc = include_str!("../doc/variable/IartHandleDetails/detail.md")]
        #[cfg(not(feature = "no-alloc"))]
        pub detail: Option<&'a Box<ErrorDetail>>,
        #[cfg(feature = "no-alloc")]
        #[doc = include_str!("../doc/variable/IartHandleDetails/detail.md")]
        pub detail: Option<&'a ErrorDetail>,

        pub is_err: Option<bool>,

        #[doc = include_str!("../doc/variable/IartHandleDetails/log.md")]
        #[cfg(all(feature = "allow-backtrace-logging", not(feature = "no-alloc")))]
        pub log: Option<&'a VecDeque<&'static Location<'static>>>,
        #[doc = include_str!("../doc/variable/IartHandleDetails/log.md")]
        #[cfg(all(feature = "allow-backtrace-logging", feature = "no-alloc"))]
        pub log: Option<&'a [Option<&'static Location<'static>>]>,
    }

    #[must_use]
    #[doc = include_str!("../doc/structs/Iart.md")]
    pub struct Iart<Item = ()> {
        #[doc = include_str!("../doc/variable/Iart/handled.md")]
        pub(crate) handled: bool,

        #[doc = include_str!("../doc/variable/Iart/data.md")]
        #[cfg(not(feature = "no-alloc"))]
        pub(crate) data: Option<Result<Item, Box<ErrorDetail>>>,
        #[cfg(feature = "no-alloc")]
        #[doc = include_str!("../doc/variable/Iart/data.md")]
        pub(crate) data: Option<Result<Item, ErrorDetail>>,

        #[cfg(feature = "error-can-have-item")]
        #[doc = include_str!("../doc/variable/Iart/err_item.md")]
        pub(crate) err_item: Option<Item>,

        #[cfg(all(feature = "allow-backtrace-logging", not(feature = "no-alloc")))]
        #[doc = include_str!("../doc/variable/Iart/log.md")]
        pub(crate) log: Option<VecDeque<&'static Location<'static>>>,

        #[cfg(all(feature = "allow-backtrace-logging", feature = "no-alloc"))]
        #[doc = include_str!("../doc/variable/Iart/log.md")]
        pub(crate) log: Option<[Option<&'static Location<'static>>; BACK_TRACE_MAX]>,

        #[cfg(not(feature = "no-alloc"))]
        #[doc = include_str!("../doc/variable/global/trans_fns.md")]
        pub(crate) trans_fns: Option<(
            unsafe fn(Box<dyn IartErr + Send + Sync>) -> Box<dyn core::any::Any + Send + Sync>,
            unsafe fn(Box<dyn core::any::Any + Send + Sync>) -> Box<dyn IartErr + Send + Sync>,
        )>,
        #[cfg(feature = "no-alloc")]
        #[doc = include_str!("../doc/variable/global/trans_fns.md")]
        pub(crate) trans_fns: Option<(
            unsafe fn(
                &'static (dyn IartErr + Send + Sync),
            ) -> &'static (dyn core::any::Any + Send + Sync),
            unsafe fn(
                &'static (dyn core::any::Any + Send + Sync),
            ) -> &'static (dyn IartErr + Send + Sync),
        )>,
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
        #[cfg(not(feature = "no-alloc"))]
        fn clone_box_in<'a>(&self, alloc: A) -> Box<dyn IartErr<A> + 'a + Send + Sync, A>
        where
            Self: 'a;
    }

    #[cfg(not(feature = "for-nightly-error-generic-member-access"))]
    #[must_use]
    #[doc = include_str!("../doc/trait/IartErr.md")]
    pub trait IartErr<A: Allocator + Clone = alloc::alloc::Global>: Debug + Display {
        #[cfg(not(feature = "no-alloc"))]
        fn clone_box_in<'a>(&self, alloc: A) -> Box<dyn IartErr<A> + 'a + Send + Sync, A>
        where
            Self: 'a;
    }

    pub type IartLogger<A = alloc::alloc::Global> =
        for<'a, 'b> fn(event: IartEvent<'a, 'b>, iart: IartHandleDetails<A>) -> core::fmt::Result;

    #[allow(unused)]
    #[doc = include_str!("../doc/structs/IartHandleDetails.md")]
    #[derive(Clone, Debug)]
    pub struct IartHandleDetails<'a, A: alloc::alloc::Allocator + Clone = alloc::alloc::Global> {
        #[doc = include_str!("../doc/variable/IartHandleDetails/detail.md")]
        pub detail: Option<&'a Box<ErrorDetail<A>, A>>,

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
        pub(crate) data: Option<Result<Item, Box<ErrorDetail<A>, A>>>,

        #[cfg(feature = "allow-backtrace-logging")]
        #[doc = include_str!("../doc/variable/Iart/log.md")]
        pub(crate) log: Option<VecDeque<&'static Location<'static>, A>>,

        #[cfg(feature = "error-can-have-item")]
        #[doc = include_str!("../doc/variable/Iart/err_item.md")]
        pub(crate) err_item: Option<Item>,

        #[doc = include_str!("../doc/variable/Iart/allocator.md")]
        pub(crate) allocator: A,

        #[doc = include_str!("../doc/variable/global/trans_fns.md")]
        pub(crate) trans_fns: Option<(
            unsafe fn(
                Box<dyn IartErr<A> + Send + Sync, A>,
            ) -> Box<dyn core::any::Any + Send + Sync, A>,
            unsafe fn(
                Box<dyn core::any::Any + Send + Sync, A>,
            ) -> Box<dyn IartErr<A> + Send + Sync, A>,
        )>,
    }
}

#[cfg(feature = "for-nightly-allocator-api-support")]
#[doc(inline)]
pub use api_impl::*;
#[cfg(not(feature = "for-nightly-allocator-api-support"))]
#[doc(inline)]
pub use non_api_impl::*;

#[cfg(not(feature = "no-alloc"))]
use alloc::borrow::Cow;
#[cfg(not(feature = "no-alloc"))]
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
