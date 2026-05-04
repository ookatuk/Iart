#![doc = include_str!("../../../doc/modules/impls.md")]

use crate::types::Iart;
use core::panic::Location;

impl<Item> Iart<Item> {
    #[doc = include_str!("../../../doc/fn/Iart/for_each_log.md")]
    #[cfg(feature = "alloc")]
    pub fn for_each_log<F>(&self, #[allow(unused)] mut f: F)
    where
        F: FnMut(&'static Location<'static>) -> bool,
    {
        if let Some(data) = self.data.as_ref() {
            if data.is_err() || cfg!(feature = "allow-backtrace-logging-with-ok") {
                if let Some(log) = self.log.as_ref() {
                    for loc in log.iter() {
                        let res = f(loc);
                        if res {
                            break;
                        }
                    }
                }
            }
        }
    }

    #[doc = include_str!("../../../doc/fn/Iart/for_each_log.md")]
    #[cfg(not(feature = "alloc"))]
    pub fn for_each_log<F>(&self, #[allow(unused)] mut f: F)
    where
        F: FnMut(&Option<&'static Location<'static>>) -> bool,
    {
        if let Some(data) = self.data.as_ref() {
            if data.is_err() || cfg!(feature = "allow-backtrace-logging-with-ok") {
                if let Some(log) = self.log.as_ref() {
                    for loc in log.iter() {
                        let res = f(loc);
                        if res {
                            break;
                        }
                    }
                }
            }
        }
    }
}
