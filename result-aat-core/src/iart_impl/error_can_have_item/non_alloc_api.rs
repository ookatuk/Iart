#![doc = include_str!("../../../doc/modules/non_alloc_api.md")]

use crate::types::{Iart, IartErr};
#[cfg(feature = "alloc")]
use alloc::string::String;

impl<Item> Iart<Item> {
    #[inline]
    #[allow(non_snake_case)]
    #[track_caller]
    #[cold]
    #[doc = include_str!("../../../doc/fn/Iart/non_alloc_api/Err_item.md")]
    #[cfg(feature = "alloc")]
    pub fn Err_item<ERR: IartErr + 'static>(
        error: ERR,
        desc: impl Into<Option<&'static str>>,
        item: Item,
    ) -> Iart<Item> {
        let mut err = Self::Err(error, desc);
        err.err_item = Some(item);

        err
    }

    #[inline]
    #[allow(non_snake_case)]
    #[track_caller]
    #[cold]
    #[doc = include_str!("../../../doc/fn/Iart/non_alloc_api/Err_item.md")]
    #[cfg(not(feature = "alloc"))]
    pub fn Err_item<ERR: IartErr + 'static>(
        error: &'static ERR,
        desc: impl Into<Option<&'static str>>,
        item: Item,
    ) -> Iart<Item> {
        let mut err = Self::Err(error, desc);
        err.err_item = Some(item);

        err
    }

    #[inline]
    #[allow(non_snake_case)]
    #[track_caller]
    #[cold]
    #[doc = include_str!("../../../doc/fn/Iart/non_alloc_api/Err_string_item.md")]
    #[cfg(feature = "alloc")]
    pub fn Err_string_item<ERR: IartErr + 'static>(
        error: ERR,
        desc: impl Into<Option<String>>,
        item: Item,
    ) -> Self {
        let mut err = Self::Err_string(error, desc);
        err.err_item = Some(item);

        err
    }
}
