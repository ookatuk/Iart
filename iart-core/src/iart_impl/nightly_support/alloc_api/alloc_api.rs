#![doc = include_str!("../../../../doc/modules/alloc_api.md")]

use crate::events::{AutoRequestType, IartEvent};
use crate::types::{DummyErr, ErrorDetail, Iart, IartErr, IartHandleDetails, IartLogger};
use crate::utils::{cold_path, create_trace, unlikely};
use crate::{GetErrRet, Trans};
use crate::{HANDLER, is_initialized_handler};
use alloc::alloc::Allocator;
use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::string::String;
use core::fmt::{Debug, Display, Formatter};
use core::sync::atomic::Ordering;

#[cfg(feature = "allow-backtrace-logging")]
use crate::{BACK_TRACE_MAX, TRACE_REMOVE_TYPE, TRACE_UNIQUE};
#[cfg(feature = "allow-backtrace-logging")]
use alloc::collections::VecDeque;
#[cfg(any(
    feature = "allow-backtrace-logging",
    feature = "enable-pending-tracker"
))]
use core::panic::Location;

impl<T, A> IartErr<A> for &'static T
where
    T: IartErr<A> + ?Sized + 'static,
    A: Allocator + Clone + 'static,
{
    fn clone_box_in<'a>(&self, alloc: A) -> Box<dyn IartErr<A> + 'a + Send + Sync, A>
    where
        Self: 'a,
    {
        (**self).clone_box_in(alloc)
    }
}

impl<'a, A: Allocator + Clone + 'a> Clone for Box<dyn IartErr<A> + 'a, A> {
    fn clone(&self) -> Self {
        let alloc = Box::allocator(self).clone();
        (**self).clone_box_in(alloc)
    }
}

impl<A: core::alloc::Allocator + Clone> IartErr<A> for DummyErr {
    fn clone_box_in<'a>(&self, alloc: A) -> Box<dyn IartErr<A> + 'a + Send + Sync, A>
    where
        Self: 'a,
    {
        Box::new_in(DummyErr {}, alloc)
    }
}

impl<A: alloc::alloc::Allocator + Clone> ErrorDetail<A> {
    pub fn default_in(alloc: A) -> ErrorDetail<A> {
        Self {
            ty: Some(Box::new_in(DummyErr {}, alloc)),
            desc: None,
            trans_fns: jen_fns!(DummyErr, A),
        }
    }
}

impl<A: alloc::alloc::Allocator + Clone> ErrorDetail<A> {
    #[doc = include_str!("../../../../doc/fn/ErrorDetail/new.md")]
    pub unsafe fn new(
        ty: Box<dyn IartErr<A> + Send + Sync, A>,
        desc: Option<Cow<'static, str>>,
        to_any: Trans<A>,
    ) -> Self {
        Self {
            ty: Some(ty),
            desc,
            trans_fns: to_any,
        }
    }
}

impl<A: alloc::alloc::Allocator + Clone + 'static> Clone for ErrorDetail<A> {
    fn clone(&self) -> Self {
        Self {
            ty: {
                if let Some(ty) = &self.ty {
                    let alloc = Box::allocator(&ty).clone();
                    Some(ty.clone_box_in(alloc))
                } else {
                    cold_path();
                    None
                }
            },
            desc: self.desc.clone(),
            trans_fns: self.trans_fns.clone(),
        }
    }
}

impl<A: alloc::alloc::Allocator + Clone> Display for ErrorDetail<A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "ErrorDetail")
    }
}

impl<Item, A: alloc::alloc::Allocator + Clone + 'static> Iart<Item, A> {
    #[doc = include_str!("../../../../doc/fn/Iart/send_log_to_handler.md")]
    pub(crate) fn send_log_to_handler<const NOT_RESULT_REQUIRED: bool>(
        &self,
        event: IartEvent,
    ) -> core::fmt::Result {
        if unlikely(!is_initialized_handler()) {
            return Ok(());
        }

        let ptr = HANDLER.load(Ordering::Acquire);

        if !ptr.is_null() {
            let logger: IartLogger<A> = unsafe { core::mem::transmute(ptr) };

            let detail = match self.data.as_ref() {
                Some(data) => data.as_ref().err(),
                None => None,
            };

            let details = IartHandleDetails::<A> {
                detail,
                #[cfg(feature = "allow-backtrace-logging")]
                log: self.log.as_ref(),
                is_err: if self.data.is_some() {
                    Some(self.is_err().unwrap())
                } else {
                    None
                },
            };

            let res = logger(event, details);
            if NOT_RESULT_REQUIRED { Ok(()) } else { res }
        } else {
            Ok(())
        }
    }

    #[inline]
    #[doc = include_str!("../../../../doc/fn/Iart/alloc_api/new_ok.md")]
    #[track_caller]
    pub fn new_ok(item: impl Into<Item>) -> Iart<Item, A>
    where
        A: Default,
    {
        Iart::<Item, A>::new_ok_in(item, A::default())
    }

    #[inline]
    #[track_caller]
    #[cold]
    #[doc = include_str!("../../../../doc/fn/Iart/alloc_api/new_err.md")]
    pub fn new_err<ERR: IartErr<A> + Send + Sync + 'static>(
        error: ERR,
        desc: impl Into<Option<&'static str>>,
    ) -> Iart<Item, A>
    where
        A: Default,
    {
        Iart::<Item, A>::new_err_in(error, desc, A::default())
    }

    #[doc = include_str!("../../../../doc/fn/Iart/alloc_api/new_ok_in.md")]
    #[inline]
    #[track_caller]
    pub fn new_ok_in(item: impl Into<Item>, allocator: A) -> Iart<Item, A> {
        Iart::<Item, A> {
            data: Some(Ok(())),
            handled: false,
            allocator: allocator.clone(),
            #[cfg(feature = "allow-backtrace-logging")]
            log: create_trace::<true>(allocator),
            trans_fns: None,
            item: Some(item.into()),
            #[cfg(feature = "enable-pending-tracker")]
            tracking_id: { crate::utils::add_to_tracker(Location::caller()) },
        }
    }

    #[track_caller]
    #[cold]
    #[doc = include_str!("../../../../doc/fn/Iart/alloc_api/new_err_in.md")]
    pub fn new_err_in<ERR: IartErr<A> + Send + Sync + 'static>(
        error: ERR,
        desc: impl Into<Option<&'static str>>,
        allocator: A,
    ) -> Iart<Item, A> {
        let to_any = jen_fns!(ERR, A);

        Iart::<Item, A> {
            data: Some(Err(unsafe {
                ErrorDetail::<A>::new(
                    Box::new_in(error, allocator.clone()),
                    desc.into().map(|x| Cow::Borrowed(x)),
                    to_any.clone(),
                )
            })),
            handled: false,
            allocator: allocator.clone(),
            #[cfg(feature = "allow-backtrace-logging")]
            log: create_trace::<false>(allocator),
            trans_fns: Some(to_any),
            item: None,
            #[cfg(feature = "enable-pending-tracker")]
            tracking_id: { crate::utils::add_to_tracker(Location::caller()) },
        }
    }

    #[track_caller]
    #[cold]
    #[doc = include_str!("../../../../doc/fn/Iart/alloc_api/new_string_err_in.md")]
    pub fn new_string_err_in<ERR>(
        error: ERR,
        desc: impl Into<Option<String>>,
        allocator: A,
    ) -> Iart<Item, A>
    where
        ERR: IartErr<A> + Send + Sync + 'static,
    {
        let res = {
            let to_any = jen_fns!(ERR, A);

            Self {
                data: Some(Err(unsafe {
                    ErrorDetail::new(
                        Box::new_in(error, allocator.clone()),
                        desc.into().map(|x| Cow::Owned(x)),
                        to_any.clone(),
                    )
                })),
                handled: false,
                allocator: allocator.clone(),
                #[cfg(feature = "allow-backtrace-logging")]
                log: create_trace::<false>(allocator),
                trans_fns: Some(to_any),
                item: None,
                #[cfg(feature = "enable-pending-tracker")]
                tracking_id: { crate::utils::add_to_tracker(Location::caller()) },
            }
        };

        res
    }

    #[inline]
    #[track_caller]
    #[cold]
    #[doc = include_str!("../../../../doc/fn/Iart/alloc_api/new_string_err.md")]
    pub fn new_string_err<ERR: IartErr<A> + Send + Sync + 'static>(
        error: ERR,
        desc: impl Into<Option<String>>,
    ) -> Iart<Item, A>
    where
        A: Default,
    {
        Iart::<Item, A>::new_string_err_in(error, desc, A::default())
    }

    #[inline]
    #[must_use]
    #[track_caller]
    #[doc = include_str!("../../../../doc/fn/Iart/ok.md")]
    pub fn ok(mut self) -> Result<Item, Self> {
        self.handled = true;

        self.send_log();

        unsafe {
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
            Ok(_) => {
                self.data = Some(Ok(()));
                Ok(self.item.take().unwrap())
            }
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
    pub fn err(mut self) -> Result<GetErrRet<Item, A>, Self> {
        self.handled = true;

        self.send_log();

        unsafe {
            self.send_log_to_handler::<true>(IartEvent::FunctionHook(AutoRequestType::GetErr))
                .unwrap_unchecked()
        };

        if let Some(data) = self.data.take() {
            match data {
                Ok(_) => {
                    self.handled = false;
                    self.data = Some(data);
                    Err(self)
                }
                Err(err) => Ok(GetErrRet::<Item, A> {
                    item: self.item.take(),
                    detail: err,
                }),
            }
        } else {
            cold_path();
            debug_assert!(false, "Iart: err called after consumption");
            self.handled = false;
            Err(self)
        }
    }

    #[track_caller]
    #[must_use]
    #[doc = include_str!("../../../../doc/fn/Iart/unwrap.md")]
    pub fn unwrap(mut self) -> Item
    where
        A: Debug,
    {
        self.handled = true;
        self.send_log();

        unsafe {
            self.send_log_to_handler::<true>(IartEvent::FunctionHook(AutoRequestType::UnwrapUsed))
                .unwrap_unchecked()
        };

        let data_opt = self.data.take();

        match data_opt {
            Some(Ok(_)) => self.item.take().unwrap(),
            Some(Err(_)) => {
                self.data = data_opt;
                self.expect("failed to unwrap Iart")
            }
            None => {
                cold_path();
                panic!("Iart: unwrap called after consumption");
            }
        }
    }

    #[track_caller]
    #[must_use]
    #[doc = include_str!("../../../../doc/fn/Iart/unwrap_err.md")]
    pub fn unwrap_err(mut self) -> GetErrRet<Item, A> {
        self.send_log();
        self.handled = true;

        unsafe {
            self.send_log_to_handler::<true>(IartEvent::FunctionHook(AutoRequestType::UnwrapErr))
                .unwrap_unchecked()
        };

        match self.data.take() {
            Some(Err(e)) => GetErrRet::<Item, A> {
                detail: e,
                item: self.item.take(),
            },
            Some(Ok(_)) => {
                panic!("called `Iart::unwrap_err()` on an `Ok` value");
            }
            None => {
                panic!("Iart: unwrap_err called after consumption");
            }
        }
    }

    #[inline]
    #[must_use]
    #[track_caller]
    #[doc = include_str!("../../../../doc/fn/Iart/unwrap_unchecked.md")]
    pub unsafe fn unwrap_unchecked<'a>(mut self) -> Item {
        self.handled = true;

        self.send_log();

        unsafe {
            self.send_log_to_handler::<true>(IartEvent::FunctionHook(
                AutoRequestType::UnwrapUnchecked,
            ))
            .unwrap_unchecked()
        };

        unsafe { self.item.take().unwrap_unchecked() }
    }

    #[track_caller]
    #[must_use]
    #[doc = include_str!("../../../../doc/fn/Iart/expect.md")]
    pub fn expect(mut self, msg: &str) -> Item
    where
        A: Debug,
    {
        self.handled = true;
        self.send_log();

        unsafe {
            self.send_log_to_handler::<true>(IartEvent::FunctionHook(AutoRequestType::Expect))
                .unwrap_unchecked()
        };

        match self.data.take() {
            Some(Ok(_)) => self
                .item
                .take()
                .expect("called `Iart::expect()`, but raised error"),
            Some(Err(e)) => {
                panic!("{}: {:?}", msg, e);
            }
            None => {
                panic!("{}: (Iart already consumed)", msg);
            }
        }
    }

    #[inline]
    #[track_caller]
    #[doc = include_str!("../../../../doc/fn/Iart/alloc_api/from_option_in.md")]
    pub fn from_option_in<ERR: IartErr<A> + Send + Sync + 'static>(
        data: Option<impl Into<Item>>,
        e_type: ERR,
        detail: impl Into<Option<&'static str>>,
        allocator: A,
    ) -> Iart<Item, A> {
        if let Some(item) = data {
            Iart::<Item, A>::new_ok_in(item, allocator)
        } else {
            cold_path();
            Iart::<Item, A>::new_err_in(e_type, detail, allocator)
        }
    }

    #[inline]
    #[track_caller]
    #[doc = include_str!("../../../../doc/fn/Iart/alloc_api/from_option.md")]
    pub fn from_option<ERR: IartErr<A> + Send + Sync + 'static>(
        data: Option<Item>,
        e_type: ERR,
        detail: impl Into<Option<&'static str>>,
    ) -> Iart<Item, A>
    where
        A: Default,
    {
        Self::from_option_in(data, e_type, detail, A::default())
    }

    #[track_caller]
    #[doc = include_str!("../../../../doc/fn/Iart/send_log.md")]
    pub fn send_log(&mut self) {
        #[cfg(feature = "enable-pending-tracker")]
        crate::utils::update_to_tracker(self.tracking_id, Location::caller());

        #[cfg(feature = "allow-backtrace-logging")]
        {
            if self.data.as_ref().map_or(false, |r| r.is_err()) {
                let loc = Location::caller();
                let log = match self.log.as_mut() {
                    Some(log) => log,
                    None => return,
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

                if BACK_TRACE_MAX == 0 {
                    return;
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
    #[doc = include_str!("../../../../doc/fn/Iart/map.md")]
    pub fn internal_map<F, NewItem: 'static>(mut self, fns: F) -> Iart<NewItem, A>
    where
        F: FnOnce(Item) -> NewItem,
        A: Default + Send + Sync + 'static,
    {
        let res = Iart::<NewItem, A> {
            handled: false,
            data: self.data.take(),
            item: self.item.take().map(fns),
            #[cfg(feature = "allow-backtrace-logging")]
            log: self.log.take(),
            trans_fns: self.trans_fns.take(),
            allocator: self.allocator.clone(),
            #[cfg(feature = "enable-pending-tracker")]
            tracking_id: self.tracking_id.take(),
        };

        self.handled = true;

        res
    }

    #[track_caller]
    #[inline]
    #[doc = include_str!("../../../../doc/fn/Iart/map.md")]
    pub fn map<F, NewItem: 'static>(mut self, fns: F) -> Iart<NewItem, A>
    where
        F: FnOnce(Item) -> NewItem,
        A: Default + Send + Sync + 'static,
    {
        self.send_log();

        self.send_log_to_handler::<true>(IartEvent::FunctionHook(AutoRequestType::Map))
            .unwrap();

        self.internal_map(fns)
    }
}

impl<T, A: alloc::alloc::Allocator + Clone> Debug for Iart<T, A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        self.send_log_to_handler::<false>(IartEvent::DebugRequest(f))
    }
}

impl<T, A: alloc::alloc::Allocator + Clone> Display for Iart<T, A>
where
    T: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        self.send_log_to_handler::<false>(IartEvent::DisplayRequest(f))
    }
}

impl<T, A: alloc::alloc::Allocator + Clone + 'static + Default> Default for Iart<T, A> {
    fn default() -> Self {
        let alloc = A::default();
        Iart::<T, A> {
            data: Some(Err(ErrorDetail::default_in(alloc.clone()))),
            handled: false,
            #[cfg(feature = "allow-backtrace-logging")]
            log: create_trace::<false>(alloc),
            allocator: alloc,
            item: None,
            trans_fns: Some(jen_fns!(DummyErr, A)),
            #[cfg(feature = "enable-pending-tracker")]
            tracking_id: { crate::utils::add_to_tracker(Location::caller()) },
        }
    }
}

impl<T, E: IartErr<A> + 'static + Send + Sync, A: Allocator + Clone + Default + 'static>
    From<Result<T, E>> for Iart<T, A>
{
    #[track_caller]
    fn from(res: Result<T, E>) -> Self {
        match res {
            Ok(val) => Iart::<T, A>::new_ok_in(val, A::default()),
            Err(err) => Iart::<T, A>::new_err_in(err, None, A::default()),
        }
    }
}
