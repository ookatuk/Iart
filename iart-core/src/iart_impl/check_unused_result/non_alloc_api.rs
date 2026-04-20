use crate::events::IartEvent;
use crate::types::Iart;
#[cfg(feature = "std")]
use crate::utils::unlikely;

impl<T> Drop for Iart<T> {
    fn drop(&mut self) {
        #[cfg(feature = "std")]
        if unlikely(std::thread::panicking()) {
            return;
        }

        if !self.handled {
            let is_err = self.data.as_ref().map_or(false, |r| r.is_err());
            if is_err || cfg!(feature = "check-unused-result-with-ok") {
                let _ = unsafe {
                    self.send_log_to_handler::<true>(IartEvent::DroppedWithoutCheck)
                        .unwrap_unchecked()
                };
                #[cfg(feature = "danger-allow-panic-if-unused")]
                panic!("detected unused Iart!");
            }
        }
    }
}
