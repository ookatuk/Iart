#![doc = include_str!("../../../../doc/modules/non_alloc_api.md")]

use crate::HANDLER;
use crate::events::AutoRequestType;
use crate::events::IartEvent;
use crate::is_initialized_handler;
use crate::types::IartLogger;
use crate::types::{DummyErr, ErrorDetail, Iart, IartErr, IartHandleDetails};
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
    fn clone_box(&self) -> Box<dyn IartErr + Send + Sync> {
        (**self).clone_box()
    }
}

impl Clone for Box<dyn IartErr> {
    fn clone(&self) -> Self {
        (**self).clone_box()
    }
}

impl IartErr for DummyErr {
    fn clone_box(&self) -> Box<dyn IartErr + Send + Sync> {
        Box::new(DummyErr {})
    }
}

impl Clone for ErrorDetail {
    fn clone(&self) -> Self {
        let ty: Option<Box<dyn IartErr + Send + Sync>> =
            { self.ty.as_ref().map(|b| b.clone_box()) };

        Self {
            ty,
            desc: self.desc.clone(),
            trans_fns: self.trans_fns,
        }
    }
}

impl core::fmt::Display for ErrorDetail {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "ErrorDetail")
    }
}

impl<Item> Iart<Item> {
    #[doc = include_str!("../../../../doc/fn/Iart/send_log_to_handler.md")]
    pub(crate) fn send_log_to_handler<const NO_RET_ERR: bool>(
        &self,
        event: IartEvent,
    ) -> core::fmt::Result {
        if unlikely(!is_initialized_handler()) {
            return Ok(());
        }

        let ptr = HANDLER.load(Ordering::Acquire);
        if !ptr.is_null() {
            let logger: IartLogger = unsafe { core::mem::transmute(ptr) };

            let details = IartHandleDetails {
                detail: self.data.as_ref().and_then(|r| r.as_ref().err()),
                #[cfg(feature = "allow-backtrace-logging")]
                log: self.log.as_ref(),
                is_err: if self.data.is_some() {
                    self.is_err()
                } else {
                    None
                },
            };

            let res = logger(event, details);

            if NO_RET_ERR {
                let _ = res;
                Ok(())
            } else {
                res
            }
        } else {
            Ok(())
        }
    }

    #[inline]
    #[allow(non_snake_case)]
    #[doc = include_str!("../../../../doc/fn/Iart/non_alloc_api/Ok.md")]
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
            trans_fns: None,
        }
    }

    #[inline]
    #[allow(non_snake_case)]
    #[track_caller]
    #[cold]
    #[doc = include_str!("../../../../doc/fn/Iart/non_alloc_api/Err.md")]
    pub fn Err<ERR: IartErr + 'static>(error: ERR, desc: impl Into<Option<&'static str>>) -> Self {
        let to_any = jen_fns!(ERR);

        let detail = Box::new(unsafe {
            ErrorDetail::new(Box::new(error), desc.into().map(Cow::Borrowed), to_any)
        });

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
            trans_fns: Some(to_any),
        }
    }

    #[inline]
    #[allow(non_snake_case)]
    #[track_caller]
    #[cold]
    #[doc = include_str!("../../../../doc/fn/Iart/non_alloc_api/Err_string.md")]
    pub fn Err_string<ERR: IartErr + 'static>(error: ERR, desc: impl Into<Option<String>>) -> Self {
        let to_any = jen_fns!(ERR);
        let detail = Box::new(unsafe {
            ErrorDetail::new(Box::new(error), desc.into().map(Cow::Owned), to_any)
        });

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
            trans_fns: Some(to_any),
        }
    }

    #[inline]
    #[track_caller]
    #[doc = include_str!("../../../../doc/fn/Iart/ok.md")]
    pub fn ok(mut self) -> Result<Item, Self> {
        self.handled = true;

        self.send_log();

        let _ = unsafe {
            self.send_log_to_handler::<true>(IartEvent::FunctionHook(AutoRequestType::GetOk))
                .unwrap_unchecked()
        };

        if unlikely(self.data.is_none()) {
            debug_assert!(false, "Iart: ok called after consumption");
            self.handled = false;
            return Err(self);
        }
        let data = self.data.take().unwrap();
        match data {
            Ok(item) => Ok(item),
            Err(err) => {
                cold_path();
                self.handled = false;
                self.data = Some(Err(err));
                Err(self)
            }
        }
    }

    #[inline]
    #[must_use]
    #[track_caller]
    #[doc = include_str!("../../../../doc/fn/Iart/err.md")]
    pub fn err(mut self) -> Result<(Box<ErrorDetail>, Option<Item>), Self> {
        self.handled = true;

        self.send_log();

        unsafe {
            self.send_log_to_handler::<true>(IartEvent::FunctionHook(AutoRequestType::GetErr))
                .unwrap_unchecked()
        };

        if let Some(data) = self.data.take() {
            #[cfg(feature = "error-can-have-item")]
            let item = self.err_item.take();
            #[cfg(not(feature = "error-can-have-item"))]
            let item = None;

            match data {
                Ok(_) => {
                    self.handled = false;
                    self.data = Some(data);
                    Err(self)
                }
                Err(err) => Ok((err, item)),
            }
        } else {
            cold_path();
            debug_assert!(false, "Iart: err called after consumption");
            self.handled = false;
            Err(self)
        }
    }

    #[inline]
    #[track_caller]
    #[must_use]
    #[doc = include_str!("../../../../doc/fn/Iart/unwrap.md")]
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
    #[must_use]
    #[doc = include_str!("../../../../doc/fn/Iart/expect.md")]
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
    #[must_use]
    #[doc = include_str!("../../../../doc/fn/Iart/unwrap_err.md")]
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
    #[must_use]
    #[doc = include_str!("../../../../doc/fn/Iart/unwrap_unchecked.md")]
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
    #[doc = include_str!("../../../../doc/fn/Iart/send_log.md")]
    pub fn send_log(&mut self) {
        #[cfg(feature = "allow-backtrace-logging")]
        {
            if self.data.as_ref().map_or(false, |r| r.is_err()) {
                let loc = Location::caller();
                let log = match self.log.as_mut() {
                    Some(log) => log,
                    None => {
                        return;
                    }
                };

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
    #[doc = include_str!("../../../../doc/fn/Iart/non_alloc_api/from_option.md")]
    pub fn from_option<ERR: IartErr + 'static>(
        data: Option<Item>,
        e_type: ERR,
        detail: impl Into<Option<&'static str>>,
    ) -> Self {
        if let Some(item) = data {
            Self::Ok(item)
        } else {
            cold_path();
            Self::Err(e_type, detail)
        }
    }

    #[inline]
    #[must_use]
    #[doc = include_str!("../../../../doc/fn/Iart/is_err.md")]
    pub const fn is_err(&self) -> Option<bool> {
        if let Some(data) = self.data.as_ref() {
            Some(data.is_err())
        } else {
            cold_path();
            debug_assert!(false, "Iart: is_err called after consumption");
            None
        }
    }

    #[track_caller]
    #[cfg(feature = "error-can-have-item")]
    #[doc = include_str!("../../../../doc/fn/Iart/map_err_item.md")]
    pub fn map_err_item<F, G, NewItem>(mut self, fns: F, item_fns: G) -> Iart<NewItem>
    where
        F: FnOnce(Item) -> NewItem,
        G: FnOnce(Item) -> NewItem,
    {
        self.send_log_to_handler::<true>(IartEvent::FunctionHook(AutoRequestType::Map))
            .unwrap();

        if let Some(data) = self.data.take() {
            let mut res: Iart<NewItem> = data.map(fns).into();

            #[cfg(feature = "allow-backtrace-logging")]
            {
                res.log = self.log.take();
            }

            let item = self.err_item.take();
            let item = item.map(item_fns);
            res.err_item = item;

            res.handled = false;
            self.handled = true;

            res
        } else {
            cold_path();

            let res = Iart::<NewItem> {
                handled: false,
                data: None,
                #[cfg(feature = "error-can-have-item")]
                err_item: None,
                #[cfg(feature = "allow-backtrace-logging")]
                log: self.log.take(),
                trans_fns: None,
            };

            self.handled = true;

            res
        }
    }

    #[track_caller]
    #[allow(rustdoc::broken_intra_doc_links)] // Because it should be correct but produces an error.
    #[doc = include_str!("../../../../doc/fn/Iart/map.md")]
    pub fn map<F, NewItem>(mut self, fns: F) -> Iart<NewItem>
    where
        F: FnOnce(Item) -> NewItem,
    {
        self.send_log_to_handler::<true>(IartEvent::FunctionHook(AutoRequestType::Map))
            .unwrap();

        if let Some(data) = self.data.take() {
            let mut res: Iart<NewItem> = data.map(fns).into();

            #[cfg(feature = "allow-backtrace-logging")]
            {
                res.log = self.log.take();
            }

            res.handled = false;
            self.handled = true;

            res
        } else {
            cold_path();

            let res = Iart::<NewItem> {
                handled: false,
                data: None,
                #[cfg(feature = "error-can-have-item")]
                err_item: None,
                #[cfg(feature = "allow-backtrace-logging")]
                log: self.log.take(),
                trans_fns: None,
            };

            self.handled = true;

            res
        }
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
            trans_fns: self.trans_fns,
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
            trans_fns: Some(jen_fns!(DummyErr)),
        }
    }
}

impl ErrorDetail {
    #[doc = include_str!("../../../../doc/fn/ErrorDetail/new.md")]
    pub unsafe fn new(
        ty: Box<dyn IartErr + Send + Sync>,
        desc: Option<Cow<'static, str>>,
        trans_fns: (
            unsafe fn(Box<dyn IartErr + Send + Sync>) -> Box<dyn core::any::Any + Send + Sync>,
            unsafe fn(Box<dyn core::any::Any + Send + Sync>) -> Box<dyn IartErr + Send + Sync>,
        ),
    ) -> Self {
        Self {
            ty: Some(ty),
            desc,
            trans_fns,
        }
    }
}

impl<T, E: IartErr + 'static + Send + Sync> From<Result<T, E>> for Iart<T> {
    #[track_caller]
    fn from(res: Result<T, E>) -> Self {
        match res {
            Ok(val) => Iart::Ok(val),
            Err(err) => Iart::Err(err, None),
        }
    }
}

impl<T> From<Result<T, Box<ErrorDetail>>> for Iart<T> {
    #[track_caller]
    fn from(res: Result<T, Box<ErrorDetail>>) -> Self {
        match res {
            Ok(val) => Iart::Ok(val),
            Err(err) => {
                let mut iart = Iart::Err(DummyErr {}, None);
                iart.trans_fns = Some(err.trans_fns);
                iart.data = Some(Err(err));

                iart
            }
        }
    }
}
