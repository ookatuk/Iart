#![doc = include_str!("../../../doc/modules/alloc_api.md")]

use crate::events::IartEvent;
use crate::types::Iart;

impl<T, A: Clone + alloc::alloc::Allocator> Drop for Iart<T, A> {
    #[doc = include_str!("../../../doc/fn/Iart/drop.md")]
    fn drop(&mut self) {
        #[cfg(feature = "std")]
        if std::thread::panicking() {
            return;
        }

        if !self.handled {
            let is_err = self.data.as_ref().map_or(false, |r| r.is_err());
            if is_err || cfg!(feature = "check-unused-result-with-ok") {
                self.send_log_to_handler::<true>(IartEvent::DroppedWithoutCheck)
                    .unwrap();
                #[cfg(feature = "danger-allow-panic-if-unused")]
                panic!("detected unused Iart!");
            }
        }
    }
}
