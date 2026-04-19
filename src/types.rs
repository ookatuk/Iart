#[must_use]
#[derive(Debug)]
pub struct ErrorDetail<
    #[cfg(feature = "for-nightly-allocator-api-support")] A: alloc::alloc::Allocator + Clone = alloc::alloc::Global,
> {
    #[cfg(feature = "for-nightly-allocator-api-support")]
    pub(crate) ty: Option<Box<dyn IartErr<A> + Send + Sync, A>>,
    #[cfg(not(feature = "for-nightly-allocator-api-support"))]
    pub(crate) ty: Option<Box<dyn IartErr + Send + Sync>>,

    #[cfg(feature = "for-nightly-allocator-api-support")]
    pub(crate) trans_fns: (
        fn(Box<dyn IartErr<A> + Send + Sync, A>) -> Box<dyn core::any::Any + Send + Sync, A>,
        fn(Box<dyn core::any::Any + Send + Sync, A>) -> Box<dyn IartErr<A> + Send + Sync, A>,
    ),
    #[cfg(not(feature = "for-nightly-allocator-api-support"))]
    pub(crate) trans_fns: (
        fn(Box<dyn IartErr + Send + Sync>) -> Box<dyn core::any::Any + Send + Sync>,
        fn(Box<dyn core::any::Any + Send + Sync>) -> Box<dyn IartErr + Send + Sync>,
    ),

    pub(crate) desc: Option<Cow<'static, str>>,
}

#[cfg(not(feature = "for-nightly-allocator-api-support"))]
mod _iart_types_trait_helper {
    use crate::events::IartEvent;
    use crate::types::ErrorDetail;
    use alloc::boxed::Box;
    #[cfg(feature = "allow-backtrace-logging")]
    use alloc::collections::VecDeque;
    use core::fmt::{Debug, Display};
    #[cfg(feature = "allow-backtrace-logging")]
    use core::panic::Location;

    pub type IartLogger =
        for<'a, 'b> fn(event: IartEvent<'a, 'b>, iart: IartDroppedDetails) -> core::fmt::Result;

    #[cfg(not(feature = "for-nightly-error-generic-member-access"))]
    #[must_use]
    pub trait IartErr: Debug + Display + Send + Sync {
        fn clone_box(&self) -> Box<dyn IartErr + Send + Sync>;
    }

    #[cfg(feature = "for-nightly-error-generic-member-access")]
    #[must_use]
    pub trait IartErr: Debug + Display + core::error::Error + Send + Sync {
        fn clone_box(&self) -> Box<dyn IartErr + Send + Sync>;
    }

    #[allow(unused)]
    pub struct IartDroppedDetails<'a> {
        pub detail: Option<&'a Box<ErrorDetail>>,

        #[cfg(feature = "allow-backtrace-logging")]
        pub log: Option<&'a VecDeque<&'static Location<'static>>>,
    }

    /// Iart (Infomation and Result Trace)
    #[must_use]
    pub struct Iart<Item = ()> {
        pub(crate) handled: bool,

        pub(crate) data: Option<Result<Item, Box<ErrorDetail>>>,

        #[cfg(feature = "error-can-have-item")]
        pub(crate) err_item: Option<Item>,

        #[cfg(feature = "allow-backtrace-logging")]
        pub(crate) log: Option<VecDeque<&'static Location<'static>>>,

        pub(crate) trans_fns: Option<(
            fn(Box<dyn IartErr + Send + Sync>) -> Box<dyn core::any::Any + Send + Sync>,
            fn(Box<dyn core::any::Any + Send + Sync>) -> Box<dyn IartErr + Send + Sync>,
        )>,
    }
}

#[cfg(feature = "for-nightly-allocator-api-support")]
mod _iart_types_trait_helper {
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
    pub trait IartErr<A: Allocator + Clone = alloc::alloc::Global>:
        Debug + Display + core::error::Error
    where
        Self: 'static,
    {
        fn clone_box_in<'a>(&self, alloc: A) -> Box<dyn IartErr<A> + 'a + Send + Sync, A>
        where
            Self: 'a;
    }

    #[cfg(not(feature = "for-nightly-error-generic-member-access"))]
    #[must_use]
    pub trait IartErr<A: Allocator + Clone = alloc::alloc::Global>: Debug + Display {
        fn clone_box_in<'a>(&self, alloc: A) -> Box<dyn IartErr<A> + 'a + Send + Sync, A>
        where
            Self: 'a;
    }

    pub type IartLogger<A = alloc::alloc::Global> =
        for<'a, 'b> fn(event: IartEvent<'a, 'b>, iart: IartDroppedDetails<A>) -> core::fmt::Result;

    #[allow(unused)]
    pub struct IartDroppedDetails<'a, A: alloc::alloc::Allocator + Clone> {
        pub detail: Option<&'a Box<ErrorDetail<A>, A>>,

        #[cfg(feature = "allow-backtrace-logging")]
        pub log: Option<&'a VecDeque<&'static Location<'static>, A>>,
    }

    /// Iart (Infomation and Result Trace)
    #[must_use]
    pub struct Iart<Item = (), A: alloc::alloc::Allocator + Clone + 'static = alloc::alloc::Global> {
        pub(crate) handled: bool,

        pub(crate) data: Option<Result<Item, Box<ErrorDetail<A>, A>>>,

        #[cfg(feature = "allow-backtrace-logging")]
        pub(crate) log: Option<VecDeque<&'static Location<'static>, A>>,

        #[cfg(feature = "error-can-have-item")]
        pub(crate) err_item: Option<Item>,

        pub(crate) allocator: A,
        pub(crate) trans_fns: Option<(
            fn(Box<dyn IartErr<A> + Send + Sync, A>) -> Box<dyn core::any::Any + Send + Sync, A>,
            fn(Box<dyn core::any::Any + Send + Sync, A>) -> Box<dyn IartErr<A> + Send + Sync, A>,
        )>,
    }
}

pub use _iart_types_trait_helper::*;
use alloc::borrow::Cow;
use alloc::boxed::Box;
use core::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub struct DummyErr {}

impl core::error::Error for DummyErr {}

impl Display for DummyErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "Dummy Err")
    }
}
