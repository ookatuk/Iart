#![doc = include_str!("../../../doc/modules/non_alloc_api.md")]

use crate::events::IartEvent;
use crate::types::Iart;
#[cfg(feature = "std")]
use crate::utils::unlikely;

impl<T> Drop for Iart<T> {
    #[doc = include_str!("../../../doc/fn/Iart/drop.md")]
    fn drop(&mut self) {
        #[cfg(feature = "enable-pending-tracker")]
        crate::utils::remove_to_tracker(self.tracking_id);

        #[cfg(feature = "std")]
        if unlikely(std::thread::panicking()) {
            return;
        }

        #[cfg(feature = "check-unused-result")]
        if !self.handled {
            let is_err = self.data.as_ref().map_or(false, |r| r.is_err());
            if is_err || cfg!(feature = "check-unused-result-with-ok") {
                let _ = {
                    self.send_log_to_handler::<true>(IartEvent::DroppedWithoutCheck)
                        .unwrap();
                };
                #[cfg(feature = "danger-allow-panic-if-unused")]
                panic!("detected unused Iart!");
            }
        }
    }
}
