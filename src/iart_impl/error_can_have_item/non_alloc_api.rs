use crate::types::{Iart, IartErr};
use alloc::string::String;

impl<Item> Iart<Item> {
    #[inline]
    #[allow(non_snake_case)]
    #[track_caller]
    #[cold]
    pub fn Err_item<ERR: IartErr>(
        error: &'static ERR,
        desc: Option<&'static str>,
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
    pub fn Err_string_item<ERR: IartErr>(
        error: &'static ERR,
        desc: Option<String>,
        item: Item,
    ) -> Self {
        let mut err = Self::Err_string(error, desc);
        err.err_item = Some(item);

        err
    }
}
