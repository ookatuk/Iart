use crate::events::IartEvent;
use crate::set_handler;
#[cfg(feature = "check-unused-result")]
use crate::types::DummyErr;
use crate::types::Iart;
use crate::types::IartDroppedDetails;
use crate::types::IartErr;
#[cfg(feature = "for-nightly-allocator-api-support")]
use alloc::alloc::Global;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use core::fmt::{Display, Formatter};
use spin::Mutex;

static TEST_LOG_LOCK: Mutex<()> = Mutex::new(());

#[derive(Debug, Clone)]
struct MyError;
impl Display for MyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "My custom error")
    }
}
impl IartErr for MyError {
    #[cfg(not(feature = "for-nightly-allocator-api-support"))]
    fn clone_box(&self) -> Box<dyn IartErr + Send + Sync> {
        Box::new(self.clone())
    }
    #[cfg(feature = "for-nightly-allocator-api-support")]
    fn clone_box_in<'a>(&self, alloc: Global) -> Box<dyn IartErr<Global> + 'a + Send + Sync, Global>
    where
        Self: 'a,
    {
        Box::new_in(self.clone(), alloc)
    }
}

impl core::error::Error for MyError {}

static LOG_HISTORY: Mutex<Vec<String>> = Mutex::new(Vec::new());

#[cfg(feature = "for-nightly-allocator-api-support")]
fn test_logger<A: Clone + core::alloc::Allocator>(
    event: IartEvent,
    _details: IartDroppedDetails<A>,
) -> core::fmt::Result {
    let mut history = LOG_HISTORY.lock();
    let msg = match event {
        IartEvent::DroppedWithoutCheck => "dropped".to_string(),
        IartEvent::FunctionHook(_) => "auto_request".to_string(),
        _ => "other".to_string(),
    };
    history.push(msg);
    Ok(())
}

#[cfg(not(feature = "for-nightly-allocator-api-support"))]
fn test_logger(event: IartEvent, _details: IartDroppedDetails) -> core::fmt::Result {
    let mut history = LOG_HISTORY.lock();
    let msg = match event {
        IartEvent::DroppedWithoutCheck => "dropped".to_string(),
        IartEvent::FunctionHook(_) => "auto_request".to_string(),
        _ => "other".to_string(),
    };
    history.push(msg);
    Ok(())
}

#[test]
fn test_ok_behavior() {
    let _guard = TEST_LOG_LOCK.lock();

    let w: Iart<i32> = Iart::Ok(42);
    assert!(w.is_ok());
    assert!(!w.is_err());
    assert_eq!(w.unwrap(), 42);
}

#[test]
fn test_err_behavior() {
    let _guard = TEST_LOG_LOCK.lock();

    let w: Iart<i32> = Iart::Err(&MyError, Some("something went wrong"));
    assert!(w.is_err());
    assert!(!w.is_ok());

    let err_desc = w.get_error_desc().unwrap();
    assert_eq!(err_desc.desc.unwrap(), "something went wrong");
}

#[test]
fn test_handler_invocation() {
    let _guard = TEST_LOG_LOCK.lock();
    set_handler(test_logger);
    {
        let _history = LOG_HISTORY.lock().clear();
    }

    let w: Iart<i32> = Iart::Ok(100);
    let val = w.unwrap();

    assert_eq!(val, 100);
    let history = LOG_HISTORY.lock();
    assert!(history.contains(&"auto_request".to_string()));
}

#[test]
#[cfg(feature = "allow-backtrace-logging")]
fn test_backtrace_logging() {
    let _guard = TEST_LOG_LOCK.lock();

    fn fail_point() -> Iart<i32> {
        Iart::Err(&MyError, None)
    }

    let mut w = fail_point();
    w.send_log();

    let mut locations = Vec::new();

    w.for_each_log(|loc| {
        locations.push(loc);
        false
    });

    assert!(locations.len() >= 1);
}

#[test]
#[should_panic(expected = "called `Iart::unwrap_err()` on an `Ok` value")]
fn test_unwrap_err_panic() {
    let _guard = TEST_LOG_LOCK.lock();

    let w: Iart<i32> = Iart::Ok(10);
    let _ = w.unwrap_err();
}

#[test]
fn test_from_option() {
    let _guard = TEST_LOG_LOCK.lock();

    let opt = Some(50);
    let w = Iart::from_option(opt, &MyError, None);
    assert_eq!(w.unwrap(), 50);

    let opt_none: Option<i32> = None;
    let w_err = Iart::from_option(opt_none, &MyError, Some("none error"));
    assert!(w_err.is_err());
}

#[test]
#[cfg(feature = "for-nightly-try-support")]
fn test_try_ok_flow() {
    let _guard = TEST_LOG_LOCK.lock();

    fn f() -> Iart<i32> {
        let a = Iart::Ok(10)?;
        let b = Iart::Ok(20)?;
        Iart::Ok(a + b)
    }

    let res = f();
    assert_eq!(res.unwrap(), 30);
}

#[test]
#[cfg(feature = "for-nightly-try-support")]
fn test_try_err_flow() {
    let _guard = TEST_LOG_LOCK.lock();

    fn f() -> Iart<i32> {
        let _a = Iart::Err(&MyError, None)?;
        Iart::Ok(20)
    }

    let res = f();
    assert!(res.is_err());
    #[cfg(feature = "allow-backtrace-logging")]
    assert!(!res.log.as_ref().unwrap().is_empty());
}

#[test]
fn test_no_handler() {
    let _guard = TEST_LOG_LOCK.lock();

    let w: Iart<i32> = Iart::Ok(1);
    let _ = w.unwrap();
}

#[test]
#[cfg(feature = "check-unused-result")]
fn test_drop_without_handling() {
    let _guard = TEST_LOG_LOCK.lock();

    set_handler(test_logger);
    LOG_HISTORY.lock().clear();

    {
        let _w: Iart<i32> = Iart::Err(&DummyErr {}, Some("a"));
    }

    let history = LOG_HISTORY.lock();
    assert!(history.contains(&"dropped".to_string()));
}

#[test]
#[cfg(feature = "check-unused-result-with-ok")]
fn test_drop_without_handling_ok_version() {
    let _guard = TEST_LOG_LOCK.lock();

    set_handler(test_logger);
    LOG_HISTORY.lock().clear();

    {
        let _w: Iart<i32> = Iart::Ok(5);
    }

    let history = LOG_HISTORY.lock();
    assert!(history.contains(&"dropped".to_string()));
}

#[test]
fn test_clone_iart() {
    let _guard = TEST_LOG_LOCK.lock();

    #[allow(unused_mut)]
    let mut w: Iart<u32> = Iart::Ok(10);

    let w2 = w.clone();

    assert_eq!(w2.unwrap(), 10);
}

#[test]
fn test_error_preserved() {
    let _guard = TEST_LOG_LOCK.lock();

    let w: Iart<i32> = Iart::Err(&MyError, Some("msg"));
    let err = w.get_error_desc().unwrap();

    assert_eq!(err.desc.unwrap(), "msg");
}

#[test]
#[cfg(feature = "for-nightly-allocator-api-support")]
fn test_allocator_ok() {
    let _guard = TEST_LOG_LOCK.lock();

    let w = Iart::Ok_in(42, Global);
    assert!(w.is_ok());
    assert_eq!(w.unwrap(), 42);
}

#[test]
fn test_no_drop_after_unwrap() {
    let _guard = TEST_LOG_LOCK.lock();

    set_handler(test_logger);
    LOG_HISTORY.lock().clear();

    {
        let w: Iart<i32> = Iart::Ok(5);
        let _ = w.unwrap();
    }

    let history = LOG_HISTORY.lock();
    assert!(!history.contains(&"dropped".to_string()));
}

#[test]
#[cfg(all(feature = "check-unused-result", feature = "std"))]
fn test_drop_without_handling_if_panic_raised() {
    let _guard = TEST_LOG_LOCK.lock();

    set_handler(test_logger);
    {
        let mut history = LOG_HISTORY.lock();
        history.clear();
    }

    let _ = std::panic::catch_unwind(|| {
        let _w: Iart<i32> = Iart::Err(&DummyErr {}, Some("panic test"));

        panic!("intentional panic");
    });

    let history = LOG_HISTORY.lock();
    assert!(
        !history.contains(&"dropped".to_string()),
        "Should not log 'dropped' event during panic unwinding to avoid double panic."
    );
}
