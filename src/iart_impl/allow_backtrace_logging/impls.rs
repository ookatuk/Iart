use crate::types::Iart;
use core::panic::Location;

impl<Item> Iart<Item> {
    pub fn for_each_log<F>(&self, #[allow(unused)] mut f: F)
    where
        F: FnMut(&'static Location<'static>) -> bool,
    {
        debug_assert!(
            self.data.is_some(),
            "Iart: have_warn called after consumption"
        );

        if let Some(data) = self.data.as_ref() {
            if data.is_err() || cfg!(feature = "allow-backtrace-logging-with-ok") {
                debug_assert!(
                    self.log.is_some(),
                    "Iart: for_each_log called after consumption"
                );

                if let Some(log) = self.log.as_ref() {
                    for loc in log.iter() {
                        let res = f(loc);
                        if res {
                            break;
                        }
                    }
                }
            }
        }
    }
}
