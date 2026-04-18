use crate::types::{DummyErr, ErrorDetail, Iart};
use crate::utils::cold_path;
use alloc::boxed::Box;

impl Default for ErrorDetail {
    #[inline]
    fn default() -> Self {
        Self {
            ty: Box::new(DummyErr {}),
            desc: None,
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
}

unsafe impl<T: Send> Send for Iart<T> {}
unsafe impl<T: Sync> Sync for Iart<T> {}

#[cfg(feature = "core_error-support")]
impl core_error::Error for Iart {}
