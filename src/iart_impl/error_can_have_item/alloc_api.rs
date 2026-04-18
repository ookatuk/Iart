use crate::types::{ErrorDetail, Iart, IartErr};
use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::string::String;

#[cfg(feature = "allow-backtrace-logging")]
use alloc::collections::VecDeque;
#[cfg(feature = "allow-backtrace-logging")]
use core::panic::Location;

impl<Item, A: alloc::alloc::Allocator + Clone + 'static> Iart<Item, A> {
    #[inline]
    #[allow(non_snake_case)]
    #[track_caller]
    #[cold]
    pub fn Err_item_in(
        error: &'static (dyn IartErr<A> + Send + Sync),
        desc: Option<&'static str>,
        item: Item,
        allocator: A,
    ) -> Iart<Item, A> {
        let mut err = Self::Err_in(error, desc, allocator);
        err.err_item = Some(item);

        err
    }

    #[inline]
    #[allow(non_snake_case)]
    #[track_caller]
    #[cold]
    pub fn Err_item(
        error: &'static (dyn IartErr<A> + Send + Sync),
        desc: Option<&'static str>,
        item: Item,
    ) -> Iart<Item, A>
    where
        A: Default,
    {
        Self::Err_item_in(error, desc, item, A::default())
    }

    #[allow(non_snake_case)]
    #[track_caller]
    #[cold]
    pub fn Err_string_item_in(
        error: &'static (dyn IartErr<A> + Send + Sync),
        desc: Option<String>,
        item: Item,
        allocator: A,
    ) -> Iart<Item, A> {
        #[cfg(not(feature = "allow-backtrace-logging"))]
        let res = Self {
            data: Some(Err(Box::new_in(
                ErrorDetail::new(
                    Box::new_in(error, allocator.clone()),
                    desc.map(|x| Cow::Owned(x)),
                ),
                allocator.clone(),
            ))),
            handled: false,
            allocator: allocator,
            err_item: Some(item),
        };

        #[cfg(feature = "allow-backtrace-logging")]
        let res = {
            let mut log = VecDeque::new_in(allocator.clone());
            log.push_back(Location::caller());
            Self {
                data: Some(Err(Box::new_in(
                    ErrorDetail::new(
                        Box::new_in(error, allocator.clone()),
                        desc.map(|x| Cow::Owned(x)),
                    ),
                    allocator.clone(),
                ))),
                handled: false,
                log: Some(log),
                allocator,
                err_item: Some(item),
            }
        };

        res
    }

    #[inline]
    #[allow(non_snake_case)]
    #[track_caller]
    #[cold]
    pub fn Err_string_item(
        error: &'static (dyn IartErr<A> + Send + Sync),
        desc: Option<String>,
        item: Item,
    ) -> Iart<Item, A>
    where
        A: Default,
    {
        Iart::<Item, A>::Err_string_item_in(error, desc, item, A::default())
    }
}
