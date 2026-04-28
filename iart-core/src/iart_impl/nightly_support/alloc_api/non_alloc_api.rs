#![doc = include_str!("../../../../doc/modules/non_alloc_api.md")]

use crate::Trans;
use crate::events::AutoRequestType;
use crate::events::IartEvent;
use crate::is_initialized_handler;
use crate::types::IartLogger;
use crate::types::{DummyErr, ErrorDetail, Iart, IartErr, IartHandleDetails};
use crate::utils::cold_path;
use crate::utils::unlikely;
use crate::{GetErrRet, HANDLER};
#[cfg(feature = "alloc")]
use alloc::borrow::Cow;
#[cfg(feature = "alloc")]
use alloc::boxed::Box;
#[cfg(feature = "alloc")]
use alloc::string::String;
use core::fmt::Debug;
use core::fmt::Display;
use core::fmt::Formatter;
use core::sync::atomic::Ordering;

#[cfg(feature = "allow-backtrace-logging")]
use crate::{BACK_TRACE_MAX, TRACE_REMOVE_TYPE, TRACE_UNIQUE};
#[cfg(all(feature = "allow-backtrace-logging", feature = "alloc"))]
use alloc::collections::VecDeque;
#[cfg(feature = "allow-backtrace-logging")]
use core::panic::Location;

impl<'t, T: IartErr + ?Sized + 't> IartErr for &'t T {
    #[cfg(feature = "alloc")]
    fn clone_box(&self) -> Box<dyn IartErr + Send + Sync> {
        (**self).clone_box()
    }
}

#[cfg(feature = "alloc")]
impl Clone for Box<dyn IartErr> {
    fn clone(&self) -> Self {
        (**self).clone_box()
    }
}

impl IartErr for DummyErr {
    #[cfg(feature = "alloc")]
    fn clone_box(&self) -> Box<dyn IartErr + Send + Sync> {
        Box::new(DummyErr {})
    }
}

impl Clone for ErrorDetail {
    #[cfg(feature = "alloc")]
    fn clone(&self) -> Self {
        let ty: Option<Box<dyn IartErr + Send + Sync>> =
            { self.ty.as_ref().map(|b| b.clone_box()) };

        Self {
            ty,
            desc: self.desc.clone(),
            trans_fns: self.trans_fns,
        }
    }
    #[cfg(not(feature = "alloc"))]
    fn clone(&self) -> Self {
        Self {
            ty: self.ty,
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
    #[cfg(feature = "alloc")]
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

    #[doc = include_str!("../../../../doc/fn/Iart/send_log_to_handler.md")]
    #[cfg(not(feature = "alloc"))]
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
                log: self
                    .log
                    .as_ref()
                    .map(|l| l as &[Option<&'static Location<'static>>]),
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
    #[cfg(feature = "alloc")]
    #[doc = include_str!("../../../../doc/fn/Iart/non_alloc_api/new_ok.md")]
    pub fn new_ok(item: Item) -> Self {
        Self {
            data: Some(Ok(())),
            handled: false,
            #[cfg(feature = "allow-backtrace-logging")]
            log: {
                #[allow(unused_mut)]
                let mut log = VecDeque::new();
                #[cfg(feature = "allow-backtrace-logging-with-ok")]
                log.push_back(Location::caller());
                Some(log)
            },
            trans_fns: None,
            item: Some(item),
            #[cfg(feature = "enable-pending-tracker")]
            tracking_id: { crate::utils::add_to_tracker(Location::caller()) },
        }
    }

    #[inline]
    #[allow(non_snake_case)]
    #[cfg(not(feature = "alloc"))]
    #[doc = include_str!("../../../../doc/fn/Iart/non_alloc_api/new_ok.md")]
    pub fn new_ok(item: Item) -> Self {
        Self {
            data: Some(Ok(())),
            handled: false,
            #[cfg(feature = "allow-backtrace-logging")]
            log: {
                #[allow(unused_mut)]
                let mut log = [None; BACK_TRACE_MAX];
                #[cfg(feature = "allow-backtrace-logging-with-ok")]
                {
                    log[0] = Some(Location::caller());
                }
                Some(log)
            },
            trans_fns: None,
            item: Some(item),
            #[cfg(feature = "enable-pending-tracker")]
            tracking_id: { crate::utils::add_to_tracker(Location::caller()) },
        }
    }

    #[inline]
    #[allow(non_snake_case)]
    #[track_caller]
    #[cold]
    #[doc = include_str!("../../../../doc/fn/Iart/non_alloc_api/new_err.md")]
    #[cfg(feature = "alloc")]
    pub fn new_err<ERR: IartErr + 'static>(
        error: ERR,
        desc: impl Into<Option<&'static str>>,
    ) -> Self {
        let to_any = jen_fns!(ERR);

        let detail =
            unsafe { ErrorDetail::new(Box::new(error), desc.into().map(Cow::Borrowed), to_any) };

        Self {
            data: Some(Err(detail)),
            handled: false,
            #[cfg(feature = "allow-backtrace-logging")]
            log: {
                let mut log = VecDeque::new();
                log.push_back(Location::caller());
                Some(log)
            },
            item: None,
            trans_fns: Some(to_any),
            #[cfg(feature = "enable-pending-tracker")]
            tracking_id: { crate::utils::add_to_tracker(Location::caller()) },
        }
    }

    #[inline]
    #[allow(non_snake_case)]
    #[track_caller]
    #[cold]
    #[doc = include_str!("../../../../doc/fn/Iart/non_alloc_api/new_err.md")]
    #[cfg(not(feature = "alloc"))]
    pub fn new_err<ERR: IartErr + 'static>(
        error: &'static ERR,
        desc: impl Into<Option<&'static str>>,
    ) -> Self {
        let to_any = jen_fns!(ERR);

        let detail = unsafe { ErrorDetail::new(error, desc.into(), to_any) };

        Self {
            data: Some(Err(detail)),
            handled: false,
            #[cfg(feature = "allow-backtrace-logging")]
            log: {
                let mut log = [None; BACK_TRACE_MAX];
                log[0] = Some(Location::caller());
                Some(log)
            },
            item: None,
            trans_fns: Some(to_any),
            #[cfg(feature = "enable-pending-tracker")]
            tracking_id: { crate::utils::add_to_tracker(Location::caller()) },
        }
    }

    #[inline]
    #[allow(non_snake_case)]
    #[track_caller]
    #[cold]
    #[doc = include_str!("../../../../doc/fn/Iart/non_alloc_api/new_string_err.md")]
    #[cfg(feature = "alloc")]
    pub fn new_string_err<ERR: IartErr + 'static>(
        error: ERR,
        desc: impl Into<Option<String>>,
    ) -> Self {
        let to_any = jen_fns!(ERR);
        let detail =
            unsafe { ErrorDetail::new(Box::new(error), desc.into().map(Cow::Owned), to_any) };

        Self {
            data: Some(Err(detail)),
            handled: false,
            #[cfg(feature = "allow-backtrace-logging")]
            log: {
                let mut log = VecDeque::new();
                log.push_back(Location::caller());
                Some(log)
            },
            item: None,
            trans_fns: Some(to_any),
            #[cfg(feature = "enable-pending-tracker")]
            tracking_id: { crate::utils::add_to_tracker(Location::caller()) },
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
            Ok(_) => Ok(self.item.take().unwrap()),
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
    pub fn err(mut self) -> Result<GetErrRet<Item>, Self> {
        self.handled = true;

        self.send_log();

        unsafe {
            self.send_log_to_handler::<true>(IartEvent::FunctionHook(AutoRequestType::GetErr))
                .unwrap_unchecked()
        };

        if let Some(data) = self.data.take() {
            let item = self.item.take();

            match data {
                Ok(_) => {
                    self.handled = false;
                    self.item = item;
                    Err(self)
                }
                Err(err) => Ok(GetErrRet { item, detail: err }),
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
            Some(Ok(_)) => {
                self.data = Some(Ok(()));
                self.item.take().unwrap()
            }
            Some(Err(e)) => {
                self.data = Some(Err(e));
                self.expect("failed to unwrap Iart")
            }
            None => panic!("Iart: unwrap called after consumption"),
        }
    }

    #[track_caller]
    #[must_use]
    #[doc = include_str!("../../../../doc/fn/Iart/unwrap_err.md")]
    pub fn unwrap_err(mut self) -> GetErrRet<Item> {
        self.handled = true;

        self.send_log();

        let _ = unsafe {
            self.send_log_to_handler::<true>(IartEvent::FunctionHook(AutoRequestType::UnwrapErr))
                .unwrap_unchecked()
        };

        match self.data.take() {
            Some(Err(e)) => GetErrRet {
                detail: e,
                item: self.item.take(),
            },
            Some(Ok(_)) => panic!("called `Iart::unwrap_err()` on an `Ok` value"),
            None => panic!("Iart: unwrap_err called after consumption"),
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
            Some(Ok(_)) => {
                self.data = Some(Ok(()));
                self.item
                    .take()
                    .expect("expect called `Iart::expect()`, but raised error.")
            }
            Some(Err(e)) => panic!("{}: {:?}", msg, e),
            None => panic!("{}: (already consumed)", msg),
        }
    }

    #[inline]
    #[track_caller]
    #[must_use]
    #[doc = include_str!("../../../../doc/fn/Iart/unwrap_unchecked.md")]
    pub unsafe fn unwrap_unchecked(mut self) -> Item {
        self.handled = true;

        self.send_log();

        self.send_log_to_handler::<true>(IartEvent::FunctionHook(AutoRequestType::UnwrapUnchecked))
            .unwrap();

        unsafe { self.item.take().unwrap_unchecked() }
    }

    #[track_caller]
    #[doc = include_str!("../../../../doc/fn/Iart/send_log.md")]
    #[cfg(feature = "alloc")]
    pub fn send_log(&mut self) {
        #[cfg(feature = "enable-pending-tracker")]
        crate::utils::update_to_tracker(self.tracking_id, Location::caller());

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

    #[track_caller]
    #[doc = include_str!("../../../../doc/fn/Iart/send_log.md")]
    #[cfg(not(feature = "alloc"))]
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
                    let data = log.iter().rev();

                    for i in data {
                        if let Some(back) = i {
                            if back.file() == loc.file()
                                && back.line() == loc.line()
                                && back.column() == loc.column()
                            {
                                return;
                            } else {
                                break;
                            }
                        }
                    }
                }

                if log.len() >= BACK_TRACE_MAX {
                    match TRACE_REMOVE_TYPE {
                        "first" => return,
                        "last" => {
                            log.copy_within(1..BACK_TRACE_MAX, 0);
                        }
                        "good" => {
                            if BACK_TRACE_MAX > 2 {
                                log.copy_within(2..BACK_TRACE_MAX, 1);
                            } else {
                                log.copy_within(1..BACK_TRACE_MAX, 0);
                            }
                        }
                        _ => {}
                    }
                    log[BACK_TRACE_MAX - 1] = Some(loc);
                }
            }
        }
    }

    #[inline]
    #[track_caller]
    #[doc = include_str!("../../../../doc/fn/Iart/non_alloc_api/from_option.md")]
    #[cfg(feature = "alloc")]
    pub fn from_option<ERR: IartErr + 'static>(
        data: Option<Item>,
        e_type: ERR,
        detail: impl Into<Option<&'static str>>,
    ) -> Self {
        if let Some(item) = data {
            Self::new_ok(item)
        } else {
            cold_path();
            Self::new_err(e_type, detail)
        }
    }

    #[inline]
    #[track_caller]
    #[doc = include_str!("../../../../doc/fn/Iart/non_alloc_api/from_option.md")]
    #[cfg(not(feature = "alloc"))]
    pub fn from_option<ERR: IartErr + 'static>(
        data: Option<Item>,
        e_type: &'static ERR,
        detail: impl Into<Option<&'static str>>,
    ) -> Self {
        if let Some(item) = data {
            Self::new_ok(item)
        } else {
            cold_path();
            Self::new_err(e_type, detail)
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
    #[allow(rustdoc::broken_intra_doc_links)] // Because it should be correct but produces an error.
    #[doc = include_str!("../../../../doc/fn/Iart/map.md")]
    #[inline]
    pub fn map<F, NewItem>(mut self, fns: F) -> Iart<NewItem>
    where
        F: FnOnce(Item) -> NewItem,
    {
        self.send_log();

        self.send_log_to_handler::<true>(IartEvent::FunctionHook(AutoRequestType::Map))
            .unwrap();

        self.internal_map(fns)
    }

    #[track_caller]
    #[allow(rustdoc::broken_intra_doc_links)] // Because it should be correct but produces an error.
    #[doc = include_str!("../../../../doc/fn/Iart/map.md")]
    #[inline]
    pub fn internal_map<F, NewItem>(mut self, fns: F) -> Iart<NewItem>
    where
        F: FnOnce(Item) -> NewItem,
    {
        let res = Iart::<NewItem> {
            handled: false,
            data: self.data.take(),
            item: self.item.take().map(fns),
            #[cfg(feature = "allow-backtrace-logging")]
            log: self.log.take(),
            trans_fns: self.trans_fns.take(),
            #[cfg(feature = "enable-pending-tracker")]
            tracking_id: self.tracking_id.take(),
        };

        self.handled = true;

        res
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

impl<T> Default for Iart<T> {
    #[cfg(feature = "alloc")]
    fn default() -> Self {
        Self {
            data: Some(Err(ErrorDetail::default())),
            handled: false,
            #[cfg(feature = "allow-backtrace-logging")]
            log: {
                let mut log = VecDeque::new();
                log.push_back(Location::caller());
                Some(log)
            },
            trans_fns: Some(jen_fns!(DummyErr)),
            item: None,
            #[cfg(feature = "enable-pending-tracker")]
            tracking_id: { crate::utils::add_to_tracker(Location::caller()) },
        }
    }

    #[cfg(not(feature = "alloc"))]
    fn default() -> Self {
        Self {
            data: Some(Err(ErrorDetail::default())),
            handled: false,
            #[cfg(feature = "allow-backtrace-logging")]
            log: {
                let mut log = [None; BACK_TRACE_MAX];
                log[0] = Some(Location::caller());
                Some(log)
            },
            trans_fns: Some(jen_fns!(DummyErr)),
            item: None,
            #[cfg(feature = "enable-pending-tracker")]
            tracking_id: { crate::utils::add_to_tracker(Location::caller()) },
        }
    }
}

impl ErrorDetail {
    #[doc = include_str!("../../../../doc/fn/ErrorDetail/new.md")]
    #[cfg(feature = "alloc")]
    pub unsafe fn new(
        ty: Box<dyn IartErr + Send + Sync>,
        desc: Option<Cow<'static, str>>,
        trans_fns: Trans,
    ) -> Self {
        Self {
            ty: Some(ty),
            desc,
            trans_fns,
        }
    }

    #[doc = include_str!("../../../../doc/fn/ErrorDetail/new.md")]
    #[cfg(not(feature = "alloc"))]
    pub unsafe fn new(
        ty: &'static (dyn IartErr + Send + Sync),
        desc: Option<&'static str>,
        trans_fns: Trans,
    ) -> Self {
        Self {
            ty: Some(ty),
            desc,
            trans_fns,
        }
    }
}

#[cfg(feature = "alloc")]
impl<T, E: IartErr + 'static + Send + Sync> From<Result<T, E>> for Iart<T> {
    #[track_caller]
    fn from(res: Result<T, E>) -> Self {
        match res {
            Ok(val) => Iart::new_ok(val),
            Err(err) => Iart::new_err(err, None),
        }
    }
}

#[cfg(not(feature = "alloc"))]
impl<T, E: IartErr + 'static + Send + Sync> From<Result<T, &'static E>> for Iart<T> {
    #[track_caller]
    fn from(res: Result<T, &'static E>) -> Self {
        match res {
            Ok(val) => Iart::new_ok(val),
            Err(err) => Iart::new_err(err, None),
        }
    }
}

#[cfg(feature = "alloc")]
impl<T> From<Result<T, ErrorDetail>> for Iart<T> {
    #[track_caller]
    fn from(res: Result<T, ErrorDetail>) -> Self {
        match res {
            Ok(val) => Iart::new_ok(val),
            Err(err) => {
                let mut iart = Iart::new_err(DummyErr {}, None);
                iart.trans_fns = Some(err.trans_fns);
                iart.data = Some(Err(err));

                iart
            }
        }
    }
}

#[cfg(not(feature = "alloc"))]
impl<T> From<Result<T, ErrorDetail>> for Iart<T> {
    #[track_caller]
    fn from(res: Result<T, ErrorDetail>) -> Self {
        match res {
            Ok(val) => Iart::new_ok(val),
            Err(err) => {
                let mut iart = Iart::new_err(&DummyErr {}, None);
                iart.trans_fns = Some(err.trans_fns);
                iart.data = Some(Err(err));

                iart
            }
        }
    }
}
