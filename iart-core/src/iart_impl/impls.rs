use crate::IartErr;
use crate::events::AutoRequestType::{
    ToResult, ToResultFail, TryDownCastFail, TryDownCastUsed, TryUsed,
};
use crate::events::IartEvent;
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
    #[must_use]
    #[doc = include_str!("../../doc/fn/ErrorDetail/try_cast_err.md")]
    pub unsafe fn try_cast_err<T: 'static>(&mut self) -> Option<Box<T>> {
        let data = self.ty.take()?;
        let res = unsafe { (self.trans_fns.0)(data) };
        match res.downcast::<T>() {
            Ok(t) => Some(t),
            Err(item) => {
                self.ty = Some(unsafe { (self.trans_fns.1)(item) });
                None
            }
        }
    }
}

impl<Item: core::fmt::Debug> Iart<Item> {
    #[inline]
    #[must_use]
    #[doc = include_str!("../../doc/fn/Iart/is_ok.md")]
    pub const fn is_ok(&self) -> Option<bool> {
        if let Some(data) = self.data.as_ref() {
            Some(data.is_ok())
        } else {
            cold_path();
            debug_assert!(false, "Iart: is_ok called after consumption");
            None
        }
    }

    #[inline]
    #[must_use]
    #[track_caller]
    #[doc = include_str!("../../doc/fn/Iart/peak_err.md")]
    pub const fn peak_err(&self) -> Option<&Box<ErrorDetail>> {
        match self.data.as_ref() {
            Some(data) => match data {
                Ok(_) => None,
                Err(item) => Some(item),
            },
            None => {
                cold_path();
                debug_assert!(false, "Iart: have_warn called after consumption");
                None
            }
        }
    }

    #[track_caller]
    #[doc = include_str!("../../doc/fn/Iart/to_result.md")]
    pub unsafe fn to_result<T: 'static>(
        mut self,
    ) -> Result<
        (
            Result<(), (T, Box<ErrorDetail>)>,
            Option<Item>,
            Option<VecDeque<&'static Location<'static>>>,
        ),
        Self,
    >
    where
        Item: Debug,
    {
        self.send_log_to_handler::<true>(IartEvent::FunctionHook(ToResult))
            .unwrap();

        let log = self.log.take();
        #[cfg(feature = "error-can-have-item")]
        let err_item = self.err_item.take();
        #[cfg(not(feature = "error-can-have-item"))]
        let err_item = None;

        self.handled = true;

        match self.is_ok() {
            Some(false) => match unsafe { self.try_downcast::<T>() } {
                Err(mut me) => {
                    cold_path();
                    me.log = log;
                    #[cfg(feature = "error-can-have-item")]
                    {
                        me.err_item = err_item;
                    }

                    me.send_log_to_handler::<true>(IartEvent::FunctionHook(ToResultFail))
                        .unwrap();

                    me.handled = false;

                    Err(me)
                }
                Ok(item) => Ok((Err(item), err_item, log)),
            },
            Some(true) => {
                let res = unsafe { self.data.take().unwrap_unchecked().unwrap_unchecked() };
                Ok((Ok(()), Some(res), log))
            }
            None => {
                cold_path();

                self.log = log;

                #[cfg(feature = "error-can-have-item")]
                {
                    self.err_item = err_item;
                }

                self.handled = false;

                self.send_log_to_handler::<true>(IartEvent::FunctionHook(ToResultFail))
                    .unwrap();

                Err(self)
            }
        }
    }

    #[track_caller]
    #[doc = include_str!("../../doc/fn/Iart/try_downcast.md")]
    pub unsafe fn try_downcast<T: 'static>(mut self) -> Result<(T, Box<ErrorDetail>), Self>
    where
        Item: Debug,
    {
        if unlikely(self.is_ok().unwrap_or(false)) {
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

        let data = unsafe { (detail.trans_fns.0)(ty) };

        match data.downcast::<T>() {
            Err(item) => {
                cold_path();
                detail.ty = Some(unsafe { (detail.trans_fns.1)(item) });
                self.data = Some(Err(detail));
                let _ = self.send_log_to_handler::<true>(IartEvent::FunctionHook(TryDownCastFail));
                Err(self)
            }
            Ok(item) => {
                self.handled = true;

                let value: T = *item;

                Ok((value, detail))
            }
        }
    }

    #[doc(hidden)]
    #[inline(always)]
    #[doc = include_str!("../../doc/fn/Iart/__internal_send_try_used.md")]
    pub unsafe fn __internal_send_try_used(&self) -> core::fmt::Result {
        self.send_log_to_handler::<false>(IartEvent::FunctionHook(TryUsed))
    }

    #[doc(hidden)]
    #[inline(always)]
    #[doc = include_str!("../../doc/fn/Iart/__internal_take_data.md")]
    pub unsafe fn __internal_take_data(&mut self) -> Option<Result<Item, Box<ErrorDetail>>> {
        self.data.take()
    }

    #[doc(hidden)]
    #[inline(always)]
    #[doc = include_str!("../../doc/fn/Iart/__internal_take_log.md")]
    pub unsafe fn __internal_take_log(&mut self) -> Option<VecDeque<&'static Location<'static>>> {
        #[cfg(feature = "allow-backtrace-logging")]
        let res = self.log.take();
        #[cfg(not(feature = "allow-backtrace-logging"))]
        let res = None;
        res
    }

    #[doc(hidden)]
    #[inline]
    #[doc = include_str!("../../doc/fn/Iart/__internal_get_trans_fns.md")]
    pub unsafe fn __internal_get_trans_fns(
        &mut self,
    ) -> Option<(
        unsafe fn(Box<dyn IartErr + Send + Sync>) -> Box<dyn core::any::Any + Send + Sync>,
        unsafe fn(Box<dyn core::any::Any + Send + Sync>) -> Box<dyn IartErr + Send + Sync>,
    )> {
        self.trans_fns.clone()
    }

    #[doc(hidden)]
    #[inline(always)]
    #[doc = include_str!("../../doc/fn/Iart/__internal_mark_handled.md")]
    pub unsafe fn __internal_mark_handled(&mut self) {
        self.handled = true;
    }

    #[doc(hidden)]
    #[inline(always)]
    #[doc = include_str!("../../doc/fn/Iart/__internal_take_err_item.md")]
    pub unsafe fn __internal_take_err_item(&mut self) -> Option<Item> {
        #[cfg(feature = "error-can-have-item")]
        let res = self.err_item.take();
        #[cfg(not(feature = "error-can-have-item"))]
        let res = None;

        res
    }

    #[doc(hidden)]
    #[inline(always)]
    #[cfg(feature = "for-nightly-allocator-api-support")]
    #[doc = include_str!("../../doc/fn/Iart/__internal_get_allocator.md")]
    pub unsafe fn __internal_get_allocator(&self) -> Option<alloc::alloc::Global> {
        Some(self.allocator)
    }

    #[doc(hidden)]
    #[inline(always)]
    #[cfg(not(feature = "for-nightly-allocator-api-support"))]
    #[doc = include_str!("../../doc/fn/Iart/__internal_get_allocator.md")]
    pub unsafe fn __internal_get_allocator(&self) -> Option<u32> {
        None
    }

    #[doc(hidden)]
    #[inline(always)]
    #[doc = include_str!("../../doc/fn/Iart/__internal_rebuild_err.md")]
    pub unsafe fn __internal_rebuild_err(
        err: Box<ErrorDetail>,
        #[allow(unused)] log: Option<VecDeque<&'static Location<'static>>>,
        trans_fns: Option<(
            unsafe fn(Box<dyn IartErr + Send + Sync>) -> Box<dyn core::any::Any + Send + Sync>,
            unsafe fn(Box<dyn core::any::Any + Send + Sync>) -> Box<dyn IartErr + Send + Sync>,
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
