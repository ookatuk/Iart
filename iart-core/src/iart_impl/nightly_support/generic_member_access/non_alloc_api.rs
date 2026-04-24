#![doc = include_str!("../../../../doc/modules/non_alloc_api.md")]

use crate::types::{ErrorDetail, Iart};

use core::fmt::{Debug, Display};

impl core::error::Error for ErrorDetail {
    fn provide<'a>(&'a self, request: &mut core::error::Request<'a>) {
        if let Some(ty) = &self.ty {
            ty.provide(request);
        }
    }
}

impl<T> core::error::Error for Iart<T>
where
    T: Debug + Display,
{
    fn provide<'a>(&'a self, request: &mut core::error::Request<'a>) {
        #[cfg(feature = "allow-backtrace-logging")]
        request.provide_ref(&self.log);

        if let Some(data) = &self.data {
            if let Err(e) = data {
                e.provide(request);
            }
        }
    }
}
