use crate::HANDLER;
use crate::events::AutoRequestType;
use crate::events::IartEvent;
use crate::is_initialized_handler;
use crate::types::IartLogger;
use crate::types::{DummyErr, ErrorDetail, Iart, IartDroppedDetails, IartErr};
use crate::utils::cold_path;
use crate::utils::unlikely;
use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::fmt::Debug;
use alloc::fmt::Display;
use alloc::fmt::Formatter;
use alloc::string::String;
use core::sync::atomic::Ordering;

#[cfg(feature = "allow-backtrace-logging")]
use crate::{BACK_TRACE_MAX, TRACE_REMOVE_TYPE, TRACE_UNIQUE};
#[cfg(feature = "allow-backtrace-logging")]
use alloc::collections::VecDeque;
#[cfg(feature = "allow-backtrace-logging")]
use core::panic::Location;

impl<'t, T: IartErr + ?Sized + 't> IartErr for &'t T {
    fn clone_box(&self) -> Box<dyn IartErr> {
        (**self).clone_box()
    }
}

impl Clone for Box<dyn IartErr> {
    fn clone(&self) -> Self {
        (**self).clone_box()
    }
}

impl IartErr for DummyErr {
    fn clone_box(&self) -> Box<dyn IartErr> {
        Box::new(DummyErr {})
    }
}

impl<'a> IartDroppedDetails<'a> {
    #[inline]
    pub fn is_err(&self) -> bool {
        self.detail.is_some()
    }
    #[inline]
    pub fn is_ok(&self) -> bool {
        !self.is_err()
    }
}

impl ErrorDetail {
    pub fn new(ty: Box<dyn IartErr + Send + Sync>, desc: Option<Cow<'static, str>>) -> Self {
        Self { ty, desc }
    }
}

impl Clone for ErrorDetail {
    fn clone(&self) -> Self {
        Self {
            ty: self.ty.clone_box(),
            desc: self.desc.clone(),
        }
    }
}

impl core::fmt::Display for ErrorDetail {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "ErrorDetail")
    }
}

impl<Item> Iart<Item> {
    pub(crate) fn send_log_to_handler<const ERR_ON_PANIC: bool>(
        &self,
        event: IartEvent,
    ) -> core::fmt::Result {
        if unlikely(!is_initialized_handler()) {
            return Ok(());
        }

        let ptr = HANDLER.load(Ordering::Acquire);
        let logger: IartLogger = unsafe { core::mem::transmute(ptr) };

        let details = IartDroppedDetails {
            detail: self.data.as_ref().and_then(|r| r.as_ref().err()),
            #[cfg(feature = "allow-backtrace-logging")]
            log: self.log.as_ref(),
        };

        let res = logger(event, details);

        if ERR_ON_PANIC {
            #[cfg(feature = "ignore-handler-err")]
            let _ = res;
            #[cfg(not(feature = "ignore-handler-err"))]
            res.expect("failed to format Iart");
            Ok(())
        } else {
            res
        }
    }

    #[inline]
    #[allow(non_snake_case)]
    pub fn Ok(item: Item) -> Self {
        Self {
            data: Some(Ok(item)),
            handled: false,
            #[cfg(feature = "allow-backtrace-logging")]
            log: {
                #[allow(unused_mut)]
                let mut log = VecDeque::new();
                #[cfg(feature = "allow-backtrace-logging-with-ok")]
                log.push_back(Location::caller());
                Some(log)
            },
            #[cfg(feature = "error-can-have-item")]
            err_item: None,
        }
    }

    #[inline]
    #[allow(non_snake_case)]
    #[track_caller]
    #[cold]
    pub fn Err(error: &'static dyn IartErr, desc: Option<&'static str>) -> Self {
        let detail = Box::new(ErrorDetail::new(Box::new(error), desc.map(Cow::Borrowed)));
        Self {
            data: Some(Err(detail)),
            handled: false,
            #[cfg(feature = "allow-backtrace-logging")]
            log: {
                let mut log = VecDeque::new();
                log.push_back(Location::caller());
                Some(log)
            },
            #[cfg(feature = "error-can-have-item")]
            err_item: None,
        }
    }

    #[inline]
    #[allow(non_snake_case)]
    #[track_caller]
    #[cold]
    pub fn Err_string(error: &'static dyn IartErr, desc: Option<String>) -> Self {
        let detail = Box::new(ErrorDetail::new(Box::new(error), desc.map(Cow::Owned)));
        Self {
            data: Some(Err(detail)),
            handled: false,
            #[cfg(feature = "allow-backtrace-logging")]
            log: {
                let mut log = VecDeque::new();
                log.push_back(Location::caller());
                Some(log)
            },
            #[cfg(feature = "error-can-have-item")]
            err_item: None,
        }
    }

    #[inline]
    #[track_caller]
    pub fn ok(mut self) -> Option<Item> {
        self.handled = true;
        let _ = unsafe {
            self.send_log_to_handler::<true>(IartEvent::FunctionHook(AutoRequestType::GetOk))
                .unwrap_unchecked()
        };

        self.send_log();

        self.data.take()?.ok()
    }

    #[inline]
    #[must_use]
    #[track_caller]
    pub fn err(mut self) -> Option<(Box<ErrorDetail>, Option<Item>)> {
        self.handled = true;

        unsafe {
            self.send_log_to_handler::<true>(IartEvent::FunctionHook(AutoRequestType::GetErr))
                .unwrap_unchecked()
        };

        self.send_log();

        if let Some(data) = self.data.take() {
            #[cfg(feature = "error-can-have-item")]
            let item = self.err_item.take();
            #[cfg(not(feature = "error-can-have-item"))]
            let item = None;

            data.err().map(|x| (x, item))
        } else {
            cold_path();
            None
        }
    }

    #[inline]
    #[track_caller]
    pub fn unwrap(mut self) -> Item {
        self.send_log();
        self.handled = true;

        let _ = unsafe {
            self.send_log_to_handler::<true>(IartEvent::FunctionHook(AutoRequestType::Unwrap))
                .unwrap_unchecked()
        };

        match self.data.take() {
            Some(Ok(item)) => item,
            Some(Err(e)) => {
                self.data = Some(Err(e));
                self.expect("failed to unwrap Iart")
            }
            None => panic!("Iart: unwrap called after consumption"),
        }
    }

    #[track_caller]
    pub fn expect(mut self, msg: &str) -> Item {
        self.handled = true;

        self.send_log();

        let _ = unsafe {
            self.send_log_to_handler::<true>(IartEvent::FunctionHook(AutoRequestType::Expect))
                .unwrap_unchecked()
        };

        match self.data.take() {
            Some(Ok(t)) => t,
            Some(Err(e)) => panic!("{}: {:?}", msg, e),
            None => panic!("{}: (already consumed)", msg),
        }
    }

    #[track_caller]
    pub fn unwrap_err(mut self) -> (Box<ErrorDetail>, Option<Item>)
    where
        Item: Debug,
    {
        self.handled = true;

        self.send_log();

        let _ = unsafe {
            self.send_log_to_handler::<true>(IartEvent::FunctionHook(AutoRequestType::UnwrapErr))
                .unwrap_unchecked()
        };

        match self.data.take() {
            Some(Err(e)) => {
                #[cfg(feature = "error-can-have-item")]
                let item = self.err_item.take();
                #[cfg(not(feature = "error-can-have-item"))]
                let item = None;

                (e, item)
            }
            Some(Ok(t)) => panic!("called `Iart::unwrap_err()` on an `Ok` value: {:?}", t),
            None => panic!("Iart: unwrap_err called after consumption"),
        }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn unwrap_unchecked(mut self) -> Item {
        self.handled = true;

        self.send_log();

        let _ = unsafe {
            self.send_log_to_handler::<true>(IartEvent::FunctionHook(AutoRequestType::Unwrap))
                .unwrap_unchecked()
        };

        unsafe { self.data.take().unwrap_unchecked().unwrap_unchecked() }
    }

    #[track_caller]
    pub fn send_log(&mut self) {
        #[cfg(feature = "allow-backtrace-logging")]
        {
            if self.data.as_ref().map_or(false, |r| r.is_err()) {
                let loc = Location::caller();
                let log = self.log.as_mut().expect("Iart: log buffer missing");

                if TRACE_UNIQUE {
                    if let Some(back) = log.back() {
                        if back.file() == loc.file()
                            && back.line() == loc.line()
                            && back.column() == loc.column()
                        {
                            return;
                        }
                    }
                }

                if log.len() >= BACK_TRACE_MAX {
                    match TRACE_REMOVE_TYPE {
                        "first" => return,
                        "last" => {
                            log.pop_front();
                        }
                        "good" => {
                            if log.len() > 2 {
                                log.remove(1);
                            } else {
                                log.pop_front();
                            }
                        }
                        _ => {}
                    }
                }
                log.push_back(loc);
            }
        }
    }

    #[inline]
    #[track_caller]
    pub fn from_option(
        data: Option<Item>,
        e_type: &'static dyn IartErr,
        detail: Option<&'static str>,
    ) -> Self {
        if let Some(item) = data {
            Self::Ok(item)
        } else {
            cold_path();
            Self::Err(e_type, detail)
        }
    }

    pub fn has_data(&self) -> bool {
        self.data.is_some()
    }
}

impl<T: Display> Display for Iart<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        self.send_log_to_handler::<true>(IartEvent::DisplayRequest(f))
    }
}

impl<T: Debug> Debug for Iart<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        self.send_log_to_handler::<true>(IartEvent::DebugRequest(f))
    }
}

impl<Item: Clone> Clone for Iart<Item> {
    fn clone(&self) -> Self {
        let new_data = self.data.as_ref().map(|d| match d {
            Ok(item) => Ok(item.clone()),
            Err(err_detail_box) => Err(Box::new((**err_detail_box).clone())),
        });

        Self {
            handled: self.handled,
            data: new_data,
            #[cfg(feature = "allow-backtrace-logging")]
            log: self.log.clone(),
            #[cfg(feature = "error-can-have-item")]
            err_item: None,
        }
    }
}

impl<T> Default for Iart<T> {
    fn default() -> Self {
        Self {
            data: Some(Err(Box::new(ErrorDetail::default()))),
            handled: false,
            #[cfg(feature = "allow-backtrace-logging")]
            log: {
                let mut log = VecDeque::new();
                log.push_back(Location::caller());
                Some(log)
            },
            #[cfg(feature = "error-can-have-item")]
            err_item: None,
        }
    }
}
