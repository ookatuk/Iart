use crate::types::{ErrorDetail, Iart, IartErr};
use alloc::borrow::Cow;
use alloc::boxed::Box;
#[cfg(feature = "allow-backtrace-logging")]
use alloc::collections::VecDeque;
use alloc::string::String;
#[cfg(feature = "allow-backtrace-logging")]
use core::panic::Location;

impl<Item> Iart<Item> {
    #[inline]
    #[allow(non_snake_case)]
    #[track_caller]
    #[cold]
    pub fn Err_item(
        error: &'static dyn IartErr,
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
    pub fn Err_string_item(error: &'static dyn IartErr, desc: Option<String>, item: Item) -> Self {
        let detail = Box::new(ErrorDetail::new(Box::new(error), desc.map(Cow::Owned)));
        Self {
            data: Some(Err(detail)),
            handled: false,
            #[cfg(feature = "allow-backtrace-logging")]
            log: {
                let mut log = VecDeque::new();
                log.push_back(Location::caller());
                Some(log)
            },
            err_item: Some(item),
        }
    }
}
