use crate::IartErr;
use crate::events::AutoRequestType::{TryDownCastFail, TryDownCastUsed};
use crate::events::{AutoRequestType, IartEvent};
use crate::types::{DummyErr, ErrorDetail, Iart};
use crate::utils::{cold_path, unlikely};
use alloc::boxed::Box;
use alloc::collections::VecDeque;
use core::fmt::Debug;
use core::panic::Location;

impl Default for ErrorDetail {
    #[inline]
    fn default() -> Self {
        Self {
            ty: Some(Box::new(DummyErr {})),
            desc: None,
            trans_fns: jen_fns!(DummyErr),
        }
    }
}
impl ErrorDetail {
    pub fn try_cast_err<T: 'static>(&mut self) -> Option<Box<T>> {
        let data = self.ty.take()?;
        let res = (self.trans_fns.0)(data);
        if unlikely(!res.is::<T>()) {
            self.ty = Some((self.trans_fns.1)(res));
            None
        } else {
            match res.downcast::<T>() {
                Ok(t) => Some(t),
                Err(item) => {
                    self.ty = Some((self.trans_fns.1)(item));
                    None
                }
            }
        }
    }
}

impl<Item> Iart<Item> {
    #[inline]
    #[must_use]
    pub const fn is_ok(&self) -> bool {
        if let Some(data) = self.data.as_ref() {
            data.is_ok()
        } else {
            cold_path();
            debug_assert!(false, "Iart: is_ok called after consumption");
            false
        }
    }

    #[inline]
    #[must_use]
    #[track_caller]
    pub fn get_error_desc(mut self) -> Option<Box<ErrorDetail>> {
        self.handled = true;

        self.send_log();

        match self.data.take() {
            Some(data) => data.as_ref().err().cloned(),
            None => {
                cold_path();
                debug_assert!(false, "Iart: have_warn called after consumption");
                None
            }
        }
    }

    #[track_caller]
    pub fn try_downcast<T: 'static>(mut self) -> Result<(T, Box<ErrorDetail>), Self>
    where
        Item: Debug,
    {
        if unlikely(self.is_ok()) {
            return Err(self);
        }

        self.send_log();

        let _ = self.send_log_to_handler::<true>(IartEvent::FunctionHook(TryDownCastUsed));

        let data = self.data.take();

        if unlikely(data.is_none()) {
            debug_assert!(false, "Iart: try_downcast called after consumption");
            let _ = self.send_log_to_handler::<true>(IartEvent::FunctionHook(TryDownCastFail));
            return Err(self);
        }

        let mut detail = unsafe { data.unwrap().unwrap_err_unchecked() };
        let ty = detail.ty.take().unwrap();

        let data = (detail.trans_fns.0)(ty);
        let is_t = data.is::<T>();

        if unlikely(!is_t) {
            detail.ty = Some((detail.trans_fns.1)(data));
            self.data = Some(Err(detail));
            let _ = self.send_log_to_handler::<true>(IartEvent::FunctionHook(TryDownCastFail));
            return Err(self);
        }

        self.handled = true;

        let concrete_box = unsafe { data.downcast::<T>().unwrap_unchecked() };

        let value: T = *concrete_box;

        Ok((value, detail))
    }

    pub fn to_result<E: 'static>(
        mut self,
    ) -> Result<
        (
            Result<Item, Box<E>>,
            Option<VecDeque<&'static Location<'static>>>,
        ),
        Self,
    > {
        self.send_log();

        self.send_log_to_handler::<true>(IartEvent::FunctionHook(AutoRequestType::ToResult))
            .unwrap();

        if unlikely(self.data.is_none()) {
            self.send_log_to_handler::<true>(IartEvent::FunctionHook(
                AutoRequestType::ToResultFail,
            ))
            .unwrap();
            return Err(self);
        }

        let data = self.data.take().unwrap();

        self.handled = true;

        match data {
            Ok(item) => {
                let log = {
                    #[cfg(feature = "allow-backtrace-logging")]
                    let res = self.log.take();
                    #[cfg(not(feature = "allow-backtrace-logging"))]
                    let res = None;
                    res
                };
                Ok((Ok(item), log))
            }
            Err(mut err) => match err.try_cast_err::<E>() {
                None => {
                    cold_path();
                    self.handled = false;
                    self.send_log_to_handler::<true>(IartEvent::FunctionHook(
                        AutoRequestType::ToResultFail,
                    ))
                    .unwrap();
                    Err(self)
                }
                Some(err) => {
                    let log = {
                        #[cfg(feature = "allow-backtrace-logging")]
                        let res = self.log.take();
                        #[cfg(not(feature = "allow-backtrace-logging"))]
                        let res = None;
                        res
                    };

                    Ok((Err(err), log))
                }
            },
        }
    }

    #[doc(hidden)]
    #[inline(always)]
    pub fn __internal_send_try_used(&self) -> core::fmt::Result {
        self.send_log_to_handler::<false>(IartEvent::FunctionHook(TryDownCastUsed))
    }

    #[doc(hidden)]
    #[inline(always)]
    pub fn __internal_take_data(&mut self) -> Option<Result<Item, Box<ErrorDetail>>> {
        self.data.take()
    }

    #[doc(hidden)]
    #[inline(always)]
    pub fn __internal_take_log(&mut self) -> Option<VecDeque<&'static Location<'static>>> {
        #[cfg(feature = "allow-backtrace-logging")]
        let res = self.log.take();
        #[cfg(not(feature = "allow-backtrace-logging"))]
        let res = None;
        res
    }

    #[doc(hidden)]
    #[inline]
    pub fn __internal_get_trans_fns(
        &mut self,
    ) -> Option<(
        fn(Box<dyn IartErr + Send + Sync>) -> Box<dyn core::any::Any + Send + Sync>,
        fn(Box<dyn core::any::Any + Send + Sync>) -> Box<dyn IartErr + Send + Sync>,
    )> {
        self.trans_fns.clone()
    }

    #[doc(hidden)]
    #[inline(always)]
    pub fn __internal_mark_handled(&mut self) {
        self.handled = true;
    }

    #[doc(hidden)]
    #[inline(always)]
    pub fn __internal_take_err_item(&mut self) -> Option<Item> {
        #[cfg(feature = "error-can-have-item")]
        let res = self.err_item.take();
        #[cfg(not(feature = "error-can-have-item"))]
        let res = None;

        res
    }

    #[doc(hidden)]
    #[inline(always)]
    #[cfg(feature = "for-nightly-allocator-api-support")]
    pub fn __internal_get_allocator(&self) -> Option<alloc::alloc::Global> {
        Some(self.allocator)
    }

    #[doc(hidden)]
    #[inline(always)]
    #[cfg(not(feature = "for-nightly-allocator-api-support"))]
    pub fn __internal_get_allocator(&self) -> Option<u32> {
        None
    }

    #[doc(hidden)]
    #[inline(always)]
    pub fn __internal_rebuild_err(
        err: Box<ErrorDetail>,
        #[allow(unused)] log: Option<VecDeque<&'static Location<'static>>>,
        trans_fns: Option<(
            fn(Box<dyn IartErr + Send + Sync>) -> Box<dyn core::any::Any + Send + Sync>,
            fn(Box<dyn core::any::Any + Send + Sync>) -> Box<dyn IartErr + Send + Sync>,
        )>,
        #[allow(unused)] err_item: Option<Item>,
        #[cfg(feature = "for-nightly-allocator-api-support")] alloc: Option<alloc::alloc::Global>,
        #[cfg(not(feature = "for-nightly-allocator-api-support"))] _alloc: Option<u32>,
    ) -> Self {
        Self {
            handled: false,
            data: Some(Err(err)),
            #[cfg(feature = "error-can-have-item")]
            err_item,
            #[cfg(feature = "allow-backtrace-logging")]
            log,
            trans_fns,
            #[cfg(feature = "for-nightly-allocator-api-support")]
            allocator: alloc.unwrap(),
        }
    }
}

unsafe impl<T: Send> Send for Iart<T> {}
unsafe impl<T: Sync> Sync for Iart<T> {}

#[cfg(feature = "core_error-support")]
impl core_error::Error for Iart {}
