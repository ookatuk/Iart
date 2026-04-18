use crate::types::{ErrorDetail, Iart};

use core::fmt::{Debug, Display};

impl<A: alloc::alloc::Allocator + Clone + Debug> core::error::Error for ErrorDetail<A> {
    fn provide<'a>(&'a self, request: &mut core::error::Request<'a>) {
        self.ty.provide(request);
    }
}

impl<T, A: alloc::alloc::Allocator + Clone + Debug> core::error::Error for Iart<T, A>
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
