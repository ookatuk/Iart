#![doc = include_str!("../../../doc/modules/alloc_api.md")]

use crate::types::{Iart, IartErr};
use alloc::string::String;

impl<Item, A: alloc::alloc::Allocator + Clone + 'static> Iart<Item, A> {
    #[inline]
    #[allow(non_snake_case)]
    #[track_caller]
    #[cold]
    #[doc = include_str!("../../../doc/fn/Iart/alloc_api/Err_item_in.md")]
    pub fn Err_item_in<ERR: IartErr<A> + Send + Sync + 'static>(
        error: ERR,
        desc: impl Into<Option<&'static str>>,
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
    #[doc = include_str!("../../../doc/fn/Iart/alloc_api/Err_item.md")]
    pub fn Err_item<ERR: IartErr<A> + Send + Sync + 'static>(
        error: ERR,
        desc: impl Into<Option<&'static str>>,
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
    #[doc = include_str!("../../../doc/fn/Iart/alloc_api/Err_string_item_in.md")]
    pub fn Err_string_item_in<ERR: IartErr<A> + Send + Sync + 'static>(
        error: ERR,
        desc: impl Into<Option<String>>,
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
    #[doc = include_str!("../../../doc/fn/Iart/alloc_api/Err_string_item.md")]
    pub fn Err_string_item<ERR: IartErr<A> + Send + Sync + 'static>(
        error: ERR,
        desc: impl Into<Option<String>>,
        item: Item,
    ) -> Iart<Item, A>
    where
        A: Default,
    {
        Iart::<Item, A>::Err_string_item_in(error, desc, item, A::default())
    }
}
