use crate::types::{Iart, IartErr};
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
    pub fn Err_item_in<ERR: IartErr<A> + Send + Sync>(
        error: &'static ERR,
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
    pub fn Err_item<ERR: IartErr<A> + Send + Sync>(
        error: &'static ERR,
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
    #[inline]
    pub fn Err_string_item_in<ERR: IartErr<A> + Send + Sync>(
        error: &'static ERR,
        desc: Option<String>,
        item: Item,
        allocator: A,
    ) -> Iart<Item, A> {
        let mut err = Self::Err_string_in(error, desc, allocator);
        err.err_item = Some(item);
        err
    }

    #[inline]
    #[allow(non_snake_case)]
    #[track_caller]
    #[cold]
    pub fn Err_string_item<ERR: IartErr<A> + Send + Sync>(
        error: &'static ERR,
        desc: Option<String>,
        item: Item,
    ) -> Iart<Item, A>
    where
        A: Default,
    {
        Iart::<Item, A>::Err_string_item_in(error, desc, item, A::default())
    }
}
