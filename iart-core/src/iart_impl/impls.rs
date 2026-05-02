#[cfg(feature = "allow-backtrace-logging")]
use crate::IartBacktrace;
use crate::events::AutoRequestType::{ToResultUsed, TryDownCastFail, TryDownCastUsed, TryUsed};
use crate::events::IartEvent;
use crate::types::{DummyErr, ErrorDetail, Iart};
use crate::utils::{cold_path, unlikely};
use crate::{DownCasted, ToResultRet, Trans};
#[cfg(feature = "alloc")]
use alloc::boxed::Box;
use core::fmt::Debug;

impl Default for ErrorDetail {
    #[inline]
    #[cfg(feature = "alloc")]
    fn default() -> Self {
        Self {
            ty: Some(Box::new(DummyErr {})),
            desc: None,
            trans_fns: jen_fns!(DummyErr),
        }
    }

    #[inline]
    #[cfg(not(feature = "alloc"))]
    fn default() -> Self {
        Self {
            ty: Some(&DummyErr {}),
            desc: None,
            trans_fns: jen_fns!(&DummyErr),
        }
    }
}

impl ErrorDetail {
    #[must_use]
    #[doc = include_str!("../../doc/fn/ErrorDetail/try_cast_err.md")]
    #[cfg(feature = "alloc")]
    pub unsafe fn try_cast_err<T: 'static>(&mut self) -> Option<Box<T>> {
        let data = self.ty.take()?;
        let res = unsafe { (self.trans_fns.to_any)(data) };
        match res.downcast::<T>() {
            Ok(t) => Some(t),
            Err(item) => {
                self.ty = Some(unsafe { (self.trans_fns.from_any)(item) });
                None
            }
        }
    }

    #[must_use]
    #[doc = include_str!("../../doc/fn/ErrorDetail/try_cast_err.md")]
    #[cfg(not(feature = "alloc"))]
    pub unsafe fn try_cast_err<T: 'static>(&mut self) -> Option<&'static T> {
        let data = self.ty.take()?;
        let res = unsafe { (self.trans_fns.to_any)(data) };
        match res.downcast_ref::<T>() {
            Some(t) => Some(t),
            None => {
                self.ty = Some(data);
                None
            }
        }
    }
}

impl<Item> Iart<Item> {
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
    pub const fn peak_err(&self) -> Option<&ErrorDetail> {
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
    pub unsafe fn to_result<E: 'static>(mut self) -> Result<ToResultRet<E, Item>, Self>
    where
        Item: Debug,
    {
        self.send_log_to_handler::<true>(IartEvent::FunctionHook(ToResultUsed))
            .unwrap();

        if self.data.is_none() {
            cold_path();
            debug_assert!(false, "Iart: to_result called after consumption");
            return Err(self);
        }

        #[cfg(feature = "allow-backtrace-logging")]
        let log = self.log.take();

        let data = {
            let raw_data = self.data.take().unwrap();
            match raw_data {
                Ok(_) => Ok(()),
                Err(mut data) => {
                    let err = unsafe { data.try_cast_err::<E>() };
                    if err.is_none() {
                        self.data = Some(Err(data));
                        return Err(self);
                    }
                    let err = err.unwrap();
                    Err((err, data))
                }
            }
        };

        Ok(ToResultRet {
            error_data: data,
            item: self.item.take(),
            #[cfg(feature = "allow-backtrace-logging")]
            backtrace: log,
        })
    }

    #[track_caller]
    #[doc = include_str!("../../doc/fn/Iart/try_downcast.md")]
    #[cfg(feature = "alloc")]
    pub unsafe fn try_downcast<T: 'static>(mut self) -> Result<DownCasted<T>, Self>
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

        let data = unsafe { (detail.trans_fns.to_any)(ty) };

        match data.downcast::<T>() {
            Err(item) => {
                cold_path();
                detail.ty = Some(unsafe { (detail.trans_fns.from_any)(item) });
                self.data = Some(Err(detail));
                let _ = self.send_log_to_handler::<true>(IartEvent::FunctionHook(TryDownCastFail));
                Err(self)
            }
            Ok(item) => {
                self.handled = true;

                let value: T = *item;

                Ok(DownCasted {
                    downcast: value,
                    detail,
                })
            }
        }
    }

    #[track_caller]
    #[doc = include_str!("../../doc/fn/Iart/try_downcast.md")]
    #[cfg(not(feature = "alloc"))]
    pub unsafe fn try_downcast<T: 'static>(mut self) -> Result<DownCasted<T>, Self>
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

        let data = unsafe { (detail.trans_fns.to_any)(ty) };

        match data.downcast_ref::<T>() {
            None => {
                cold_path();
                detail.ty = Some(unsafe { (detail.trans_fns.from_any)(data) });
                self.data = Some(Err(detail));
                let _ = self.send_log_to_handler::<true>(IartEvent::FunctionHook(TryDownCastFail));
                Err(self)
            }
            Some(item) => {
                self.handled = true;

                Ok(DownCasted {
                    detail,
                    downcast: item,
                })
            }
        }
    }

    #[inline]
    #[doc = include_str!("../../doc/fn/Iart/with_item.md")]
    pub fn with_item(mut self, item: impl Into<Item>) -> Self {
        self.item = Some(item.into());

        self
    }

    #[doc(hidden)]
    #[inline(always)]
    #[doc = include_str!("../../doc/fn/Iart/__internal_send_try_used.md")]
    pub fn __internal_send_try_used(&self) -> core::fmt::Result {
        self.send_log_to_handler::<false>(IartEvent::FunctionHook(TryUsed))
    }

    #[doc(hidden)]
    #[inline(always)]
    #[doc = include_str!("../../doc/fn/Iart/__internal_take_data.md")]
    pub fn __internal_take_data(&mut self) -> Option<Result<(), ErrorDetail>> {
        self.data.take()
    }

    #[doc(hidden)]
    #[inline(always)]
    #[doc = include_str!("../../doc/fn/Iart/__internal_take_log.md")]
    #[cfg(feature = "allow-backtrace-logging")]
    pub fn __internal_take_log(&mut self) -> Option<IartBacktrace> {
        self.log.take()
    }

    #[doc(hidden)]
    #[inline(always)]
    #[doc = include_str!("../../doc/fn/Iart/__internal_take_log.md")]
    #[cfg(not(feature = "allow-backtrace-logging"))]
    pub fn __internal_take_log(&mut self) -> Option<i32> {
        None
    }

    #[inline]
    #[doc = include_str!("../../doc/fn/Iart/__internal_get_trans_fns.md")]
    pub unsafe fn __internal_get_trans_fns(&mut self) -> Option<Trans> {
        self.trans_fns.clone()
    }

    #[doc(hidden)]
    #[inline(always)]
    #[doc = include_str!("../../doc/fn/Iart/__internal_mark_handled.md")]
    pub fn __internal_mark_handled(&mut self) {
        self.handled = true;
    }

    #[doc(hidden)]
    #[inline(always)]
    #[cfg(feature = "for-nightly-allocator-api-support")]
    #[doc = include_str!("../../doc/fn/Iart/__internal_get_allocator.md")]
    pub fn __internal_get_allocator(&self) -> Option<alloc::alloc::Global> {
        Some(self.allocator)
    }

    #[doc(hidden)]
    #[inline(always)]
    #[cfg(not(feature = "for-nightly-allocator-api-support"))]
    #[doc = include_str!("../../doc/fn/Iart/__internal_get_allocator.md")]
    pub fn __internal_get_allocator(&self) -> Option<u32> {
        None
    }

    #[doc(hidden)]
    #[inline(always)]
    #[doc = include_str!("../../doc/fn/Iart/__internal_take_track_id.md")]
    pub fn __internal_take_track_id(&mut self) -> Option<usize> {
        #[cfg(feature = "enable-pending-tracker")]
        return self.tracking_id.take();
        #[cfg(not(feature = "enable-pending-tracker"))]
        return None;
    }

    #[doc(hidden)]
    #[inline(always)]
    #[doc = include_str!("../../doc/fn/Iart/__internal_take_item.md")]
    pub fn __internal_take_item(&mut self) -> Option<Item> {
        self.item.take()
    }
}

unsafe impl<T: Send> Send for Iart<T> {}
unsafe impl<T: Sync> Sync for Iart<T> {}

#[cfg(feature = "core_error-support")]
impl<T: core::fmt::Debug + core::fmt::Display> core_error::Error for Iart<T> {}
