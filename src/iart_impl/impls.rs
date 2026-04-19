use crate::events::AutoRequestType::{TryDownCastFail, TryDownCastUsed};
use crate::events::IartEvent;
use crate::types::{DummyErr, ErrorDetail, Iart};
use crate::utils::{cold_path, unlikely};
use alloc::boxed::Box;
use core::fmt::Debug;

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

impl<Item> Iart<Item> {
    #[inline]
    #[must_use]
    pub const fn is_err(&self) -> bool {
        if let Some(data) = self.data.as_ref() {
            data.is_err()
        } else {
            cold_path();
            debug_assert!(false, "Iart: is_err called after consumption");
            false
        }
    }
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

        let _ = self.send_log_to_handler::<true>(IartEvent::FunctionHook(TryDownCastUsed));

        self.send_log();
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

        let concrete_box = unsafe { data.downcast::<T>().unwrap_unchecked() };

        let value: T = *concrete_box;

        Ok((value, detail))
    }
}

unsafe impl<T: Send> Send for Iart<T> {}
unsafe impl<T: Sync> Sync for Iart<T> {}

#[cfg(feature = "core_error-support")]
impl core_error::Error for Iart {}
