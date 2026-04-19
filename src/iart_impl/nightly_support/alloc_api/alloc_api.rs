use crate::events::{AutoRequestType, IartEvent};
use crate::types::{DummyErr, ErrorDetail, Iart, IartDroppedDetails, IartErr, IartLogger};
use crate::utils::{cold_path, unlikely};
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
#[cfg(feature = "allow-backtrace-logging")]
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

impl<'a, A: alloc::alloc::Allocator + Clone> IartDroppedDetails<'a, A> {
    #[inline]
    pub fn is_err(&self) -> bool {
        self.detail.is_some()
    }
    #[inline]
    pub fn is_ok(&self) -> bool {
        !self.is_err()
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
    pub fn new(
        ty: Box<dyn IartErr<A> + Send + Sync, A>,
        desc: Option<Cow<'static, str>>,
        to_any: (
            fn(Box<dyn IartErr<A> + Send + Sync, A>) -> Box<dyn core::any::Any + Send + Sync, A>,
            fn(Box<dyn core::any::Any + Send + Sync, A>) -> Box<dyn IartErr<A> + Send + Sync, A>,
        ),
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
            trans_fns: self.trans_fns,
        }
    }
}

impl<A: alloc::alloc::Allocator + Clone> Display for ErrorDetail<A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "ErrorDetail")
    }
}

impl<Item, A: alloc::alloc::Allocator + Clone + 'static> Iart<Item, A> {
    pub(crate) fn send_log_to_handler<const ERR_ON_PANIC: bool>(
        &self,
        event: IartEvent,
    ) -> core::fmt::Result {
        if unlikely(!is_initialized_handler()) {
            return Ok(());
        }

        let ptr = HANDLER.load(Ordering::Acquire);

        let logger: IartLogger<A> = unsafe { core::mem::transmute(ptr) };

        let detail = match self.data.as_ref() {
            Some(data) => data.as_ref().err(),
            None => None,
        };

        let details = IartDroppedDetails::<A> {
            detail,
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
    pub fn Ok(item: Item) -> Iart<Item, A>
    where
        A: Default,
    {
        Iart::<Item, A>::Ok_in(item, A::default())
    }

    #[inline]
    #[allow(non_snake_case)]
    #[track_caller]
    #[cold]
    pub fn Err<ERR: IartErr<A> + Send + Sync + 'static>(
        error: &ERR,
        desc: Option<&'static str>,
    ) -> Iart<Item, A>
    where
        A: Default,
    {
        Iart::<Item, A>::Err_in(error, desc, A::default())
    }

    #[allow(non_snake_case)]
    pub fn Ok_in(item: Item, allocator: A) -> Iart<Item, A> {
        #[cfg(not(feature = "allow-backtrace-logging"))]
        let res = Self {
            data: Some(Ok(item)),
            handled: false,
            allocator: allocator,
            #[cfg(feature = "error-can-have-item")]
            err_item: None,
            trans_fns: None,
        };
        #[cfg(feature = "allow-backtrace-logging")]
        let res = {
            #[allow(unused_mut)]
            let mut log = VecDeque::<&'static Location<'static>, A>::new_in(allocator.clone());
            #[cfg(feature = "allow-backtrace-logging-with-ok")]
            log.push_back(Location::caller());

            Iart::<Item, A> {
                data: Some(Ok(item)),
                handled: false,
                allocator,
                log: Some(log),
                #[cfg(feature = "error-can-have-item")]
                err_item: None,
                trans_fns: None,
            }
        };
        res
    }

    #[allow(non_snake_case)]
    #[track_caller]
    #[cold]
    pub fn Err_in<ERR: IartErr<A> + Send + Sync + 'static>(
        error: &ERR,
        desc: Option<&'static str>,
        allocator: A,
    ) -> Iart<Item, A> {
        let to_any = jen_fns!(ERR, A);

        Iart::<Item, A> {
            data: Some(Err(Box::new_in(
                ErrorDetail::<A>::new(
                    error.clone_box_in(allocator.clone()),
                    desc.map(|x| Cow::Borrowed(x)),
                    to_any,
                ),
                allocator.clone(),
            ))),
            handled: false,
            allocator: allocator.clone(),
            #[cfg(feature = "error-can-have-item")]
            err_item: None,
            #[cfg(feature = "allow-backtrace-logging")]
            log: {
                let mut log = VecDeque::new_in(allocator.clone());
                log.push_back(core::panic::Location::caller());
                Some(log)
            },
            trans_fns: Some(to_any),
        }
    }

    #[allow(non_snake_case)]
    #[track_caller]
    #[cold]
    pub fn Err_string_in<ERR>(error: &ERR, desc: Option<String>, allocator: A) -> Iart<Item, A>
    where
        ERR: IartErr<A> + Send + Sync + 'static,
    {
        let res = {
            let to_any = jen_fns!(ERR, A);

            Self {
                data: Some(Err(Box::new_in(
                    ErrorDetail::new(
                        error.clone_box_in(allocator.clone()),
                        desc.map(|x| Cow::Owned(x)),
                        to_any,
                    ),
                    allocator.clone(),
                ))),
                handled: false,
                allocator: allocator.clone(),
                #[cfg(feature = "error-can-have-item")]
                err_item: None,
                #[cfg(feature = "allow-backtrace-logging")]
                log: {
                    let mut log = VecDeque::new_in(allocator);
                    log.push_back(Location::caller());
                    Some(log)
                },
                trans_fns: Some(to_any),
            }
        };

        res
    }

    #[inline]
    #[allow(non_snake_case)]
    #[track_caller]
    #[cold]
    pub fn Err_string<ERR: IartErr<A> + Send + Sync + 'static>(
        error: &ERR,
        desc: Option<String>,
    ) -> Iart<Item, A>
    where
        A: Default,
    {
        Iart::<Item, A>::Err_string_in(error, desc, A::default())
    }

    #[inline]
    #[must_use]
    #[track_caller]
    pub fn ok(mut self) -> Option<Item> {
        self.handled = true;

        unsafe {
            self.send_log_to_handler::<true>(IartEvent::FunctionHook(AutoRequestType::GetOk))
                .unwrap_unchecked()
        };

        self.send_log();

        if let Some(data) = self.data.take() {
            data.ok()
        } else {
            cold_path();
            debug_assert!(self.data.is_some(), "Iart: ok called after consumption");
            None
        }
    }

    #[inline]
    #[must_use]
    #[track_caller]
    pub fn err(mut self) -> Option<(Box<ErrorDetail<A>, A>, Option<Item>)> {
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

    #[track_caller]
    pub fn unwrap(mut self) -> Item
    where
        A: Debug,
    {
        unsafe {
            self.send_log_to_handler::<true>(IartEvent::FunctionHook(AutoRequestType::Unwrap))
                .unwrap_unchecked()
        };

        let data_opt = self.data.take();
        self.handled = true;
        self.send_log();

        match data_opt {
            Some(Ok(item)) => item,
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
    pub fn unwrap_err(mut self) -> (Box<ErrorDetail<A>, A>, Option<Item>)
    where
        Item: Debug,
    {
        self.send_log();
        self.handled = true;

        unsafe {
            self.send_log_to_handler::<true>(IartEvent::FunctionHook(AutoRequestType::UnwrapErr))
                .unwrap_unchecked()
        };

        match self.data.take() {
            Some(Err(e)) => {
                #[cfg(not(feature = "error-can-have-item"))]
                let item = None;

                #[cfg(feature = "error-can-have-item")]
                let item = self.err_item.take();

                (e, item)
            }
            Some(Ok(t)) => {
                panic!("called `Iart::unwrap_err()` on an `Ok` value: {:?}", t);
            }
            None => {
                panic!("Iart: unwrap_err called after consumption");
            }
        }
    }

    #[inline]
    #[must_use]
    #[track_caller]
    pub unsafe fn unwrap_unchecked<'a>(mut self) -> Item {
        self.handled = true;

        self.send_log();

        unsafe {
            self.send_log_to_handler::<true>(IartEvent::FunctionHook(AutoRequestType::Unwrap))
                .unwrap_unchecked()
        };

        let data = self.data.take();
        unsafe { data.unwrap_unchecked().unwrap_unchecked() }
    }

    #[track_caller]
    pub fn expect(mut self, msg: &str) -> Item
    where
        A: Debug,
    {
        self.send_log();

        unsafe {
            self.send_log_to_handler::<true>(IartEvent::FunctionHook(AutoRequestType::Expect))
                .unwrap_unchecked()
        };

        match self.data.take() {
            Some(Ok(t)) => {
                self.handled = true;
                t
            }
            Some(Err(e)) => {
                self.handled = true;
                panic!("{}: {:?}", msg, e);
            }
            None => {
                panic!("{}: (Iart already consumed)", msg);
            }
        }
    }

    #[inline]
    #[track_caller]
    pub fn from_option_in<ERR: IartErr<A> + Send + Sync + 'static>(
        data: Option<Item>,
        e_type: &ERR,
        detail: Option<&'static str>,
        allocator: A,
    ) -> Iart<Item, A> {
        if let Some(item) = data {
            Iart::<Item, A>::Ok_in(item, allocator)
        } else {
            cold_path();
            Iart::<Item, A>::Err_in(e_type, detail, allocator)
        }
    }

    #[inline]
    #[track_caller]
    pub fn from_option<ERR: IartErr<A> + Send + Sync + 'static>(
        data: Option<Item>,
        e_type: &ERR,
        detail: Option<&'static str>,
    ) -> Iart<Item, A>
    where
        A: Default,
    {
        Self::from_option_in(data, e_type, detail, A::default())
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

impl<Item: Clone, A: alloc::alloc::Allocator + Clone + 'static> Clone for Iart<Item, A> {
    fn clone(&self) -> Self {
        let alloc = self.allocator.clone();

        let new_data = self.data.as_ref().map(|d| match d {
            Ok(item) => Ok(item.clone()),
            Err(err_detail_box) => Err(Box::new_in((**err_detail_box).clone(), alloc.clone())),
        });
        Self {
            handled: self.handled,
            data: new_data,
            #[cfg(feature = "allow-backtrace-logging")]
            log: self.log.clone(),
            allocator: self.allocator.clone(),
            #[cfg(feature = "error-can-have-item")]
            err_item: None,
            trans_fns: self.trans_fns,
        }
    }
}

impl<T, A: alloc::alloc::Allocator + Clone + 'static + Default> Default for Iart<T, A> {
    fn default() -> Self {
        let alloc = A::default();

        #[cfg(feature = "allow-backtrace-logging")]
        let res = {
            let mut log = VecDeque::new_in(alloc.clone());
            log.push_back(Location::caller());

            Iart::<T, A> {
                data: Some(Err(Box::new_in(
                    ErrorDetail::default_in(alloc.clone()),
                    alloc.clone(),
                ))),
                handled: false,
                log: Some(log),
                allocator: alloc,
                #[cfg(feature = "error-can-have-item")]
                err_item: None,
                trans_fns: Some(jen_fns!(DummyErr, A)),
            }
        };
        #[cfg(not(feature = "allow-backtrace-logging"))]
        let res = Iart::<T, A> {
            data: Some(Err(Box::new_in(
                ErrorDetail::default_in(alloc.clone()),
                alloc.clone(),
            ))),
            handled: false,
            allocator: alloc,
            #[cfg(feature = "error-can-have-item")]
            err_item: None,
            trans_fns: Some(jen_fns!(DummyErr, A)),
        };

        res
    }
}
