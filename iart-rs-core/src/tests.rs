use crate::events::IartEvent;
use crate::set_handler;
use crate::types::Iart;
use crate::types::IartErr;
use crate::types::IartHandleDetails;

use core::fmt::{Display, Formatter};
use spin::Mutex;

#[cfg(feature = "for-nightly-allocator-api-support")]
use alloc::alloc::Global;

#[cfg(feature = "alloc")]
use alloc::boxed::Box;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

#[cfg(not(feature = "alloc"))]
const LOG_SIZE: usize = 64;

static TEST_LOG_LOCK: Mutex<()> = Mutex::new(());

/// [`crate::MyError`] can also be used as a substitute for this.
#[derive(Debug, Clone)]
struct MyError;
impl Display for MyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "My custom error")
    }
}

#[cfg(feature = "alloc")]
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

#[cfg(not(feature = "alloc"))]
impl IartErr for MyError {}

impl core::error::Error for MyError {}

#[cfg(feature = "alloc")]
static LOG_HISTORY: Mutex<Vec<&'static str>> = Mutex::new(Vec::new());
#[cfg(not(feature = "alloc"))]
static LOG_HISTORY: Mutex<[Option<&'static str>; LOG_SIZE]> = Mutex::new([None; LOG_SIZE]);

#[cfg(feature = "for-nightly-allocator-api-support")]
fn test_logger<A: Clone + core::alloc::Allocator>(
    event: IartEvent,
    _details: IartHandleDetails<A>,
) -> core::fmt::Result {
    let mut history = LOG_HISTORY.lock();
    let msg = match event {
        IartEvent::DroppedWithoutCheck => "dropped",
        IartEvent::FunctionHook(_) => "auto_request",
        _ => "other",
    };
    history.push(msg);
    Ok(())
}

#[cfg(not(feature = "for-nightly-allocator-api-support"))]
fn test_logger(event: IartEvent, _details: IartHandleDetails) -> core::fmt::Result {
    let mut history = LOG_HISTORY.lock();
    let msg = match event {
        IartEvent::DroppedWithoutCheck => "dropped",
        IartEvent::FunctionHook(_) => "auto_request",
        _ => "other",
    };
    #[cfg(feature = "alloc")]
    history.push(msg);
    #[cfg(not(feature = "alloc"))]
    {
        let mut target: usize = LOG_SIZE;

        for i in history.iter_mut().rev() {
            if i.is_some() {
                break;
            }
            target -= 1;
        }

        if LOG_SIZE == target {
            target -= 1
        }

        history[target] = Some(msg);
    }

    Ok(())
}

fn clear() {
    #[cfg(feature = "alloc")]
    let _ = LOG_HISTORY.lock().clear();
    #[cfg(not(feature = "alloc"))]
    {
        *LOG_HISTORY.lock() = [None; LOG_SIZE];
    }
}

#[test]
fn test_ok_behavior() {
    let _guard = TEST_LOG_LOCK.lock();

    let w: Iart<i32> = Iart::new_ok(42);
    assert!(w.is_ok().unwrap());
    assert!(!w.is_err().unwrap());
    assert_eq!(w.unwrap(), 42);
}

#[test]
fn test_handler_invocation() {
    let _guard = TEST_LOG_LOCK.lock();
    set_handler(test_logger);
    clear();

    let w: Iart<i32> = Iart::new_ok(100);
    let val = w.unwrap();

    assert_eq!(val, 100);
    let history = LOG_HISTORY.lock();

    #[cfg(feature = "alloc")]
    assert!(history.contains(&"auto_request"));
    #[cfg(not(feature = "alloc"))]
    assert!(history.contains(&Some("auto_request")));
}

#[test]
#[cfg(feature = "allow-backtrace-logging")]
fn test_backtrace_logging() {
    let _guard = TEST_LOG_LOCK.lock();

    #[cfg(feature = "alloc")]
    fn fail_point() -> Iart<i32> {
        Iart::new_err(MyError, None)
    }
    #[cfg(not(feature = "alloc"))]
    fn fail_point() -> Iart<i32> {
        Iart::new_err(&MyError, None)
    }

    let mut w = fail_point();
    w.send_log();

    let mut len = 0;

    w.for_each_log(|_| {
        len += 1;
        false
    });

    assert!(len >= 1);
    let _ = w.unwrap_err();
}

#[test]
#[should_panic(expected = "called `Iart::unwrap_err()` on an `Ok` value")]
fn test_unwrap_err_panic() {
    let _guard = TEST_LOG_LOCK.lock();

    let w: Iart<i32> = Iart::new_ok(10);
    let _ = w.unwrap_err();
}

#[test]
fn test_from_option() {
    let _guard = TEST_LOG_LOCK.lock();

    let opt = Some(50);

    #[cfg(feature = "alloc")]
    let w: Iart<i32> = Iart::from_option(opt, MyError, None);
    #[cfg(not(feature = "alloc"))]
    let w: Iart<i32> = Iart::from_option(opt, &MyError, None);

    assert_eq!(w.unwrap(), 50);

    let opt_none: Option<i32> = None;

    #[cfg(feature = "alloc")]
    let w_err: Iart<i32> = Iart::from_option(opt_none, MyError, Some("none error"));
    #[cfg(not(feature = "alloc"))]
    let w_err: Iart<i32> = Iart::from_option(opt_none, &MyError, None);

    assert!(w_err.is_err().unwrap());
    let _ = w_err.unwrap_err();
}

#[test]
#[cfg(feature = "for-nightly-try-support")]
fn test_try_ok_flow() {
    let _guard = TEST_LOG_LOCK.lock();

    fn f() -> Iart<i32> {
        let a: i32 = Iart::new_ok(10)?;
        let b: i32 = Iart::new_ok(20)?;
        Iart::new_ok(a + b)
    }

    let res = f();
    assert_eq!(res.unwrap(), 30);
}

#[test]
#[cfg(feature = "for-nightly-try-support")]
fn test_try_err_flow() {
    let _guard = TEST_LOG_LOCK.lock();

    #[cfg(feature = "alloc")]
    fn f() -> Iart<i32> {
        let _ = Iart::new_err(MyError, Some("test"))?;
        unreachable!();
    }
    #[cfg(not(feature = "alloc"))]
    fn f() -> Iart<i32> {
        let _ = Iart::new_err(&MyError, Some("test"))?;
        unreachable!();
    }

    let res = f();
    assert!(res.is_err().unwrap());
    #[cfg(feature = "allow-backtrace-logging")]
    assert!(!res.log.as_ref().unwrap().is_empty());

    let _ = res.unwrap_err();
}

#[test]
fn test_no_handler() {
    let _guard = TEST_LOG_LOCK.lock();

    let w: Iart<i32> = Iart::new_ok(1);
    let _ = w.unwrap();
}

#[test]
#[cfg(all(
    feature = "check-unused-result",
    not(feature = "danger-allow-panic-if-unused")
))]
fn test_drop_without_handling() {
    let _guard = TEST_LOG_LOCK.lock();

    set_handler(test_logger);
    clear();

    {
        #[cfg(feature = "alloc")]
        let _w: Iart<i32> = Iart::new_err(MyError, "a");
        #[cfg(not(feature = "alloc"))]
        let _w: Iart<i32> = Iart::new_err(&MyError, "a");
    }

    let history = LOG_HISTORY.lock();
    #[cfg(feature = "alloc")]
    assert!(history.contains(&"dropped"));
    #[cfg(not(feature = "alloc"))]
    assert!(history.contains(&Some("dropped")));
}

#[test]
#[cfg(all(
    feature = "check-unused-result-with-ok",
    not(feature = "danger-allow-panic-if-unused")
))]
fn test_drop_without_handling_ok_version() {
    let _guard = TEST_LOG_LOCK.lock();

    set_handler(test_logger);
    clear();

    {
        let _w: Iart<i32> = Iart::new_ok(5);
    }

    let history = LOG_HISTORY.lock();

    #[cfg(feature = "alloc")]
    assert!(history.contains(&"dropped"));
    #[cfg(not(feature = "alloc"))]
    assert!(history.contains(&Some("dropped")));
}

#[test]
#[should_panic(expected = "detected unused Iart!")]
#[cfg(feature = "danger-allow-panic-if-unused")]
fn test_drop_without_handling_ok_version() {
    let _guard = TEST_LOG_LOCK.lock();
    let _w: Iart<i32> = Iart::new_ok(5);
}

#[test]
fn test_error_preserved() {
    let _guard = TEST_LOG_LOCK.lock();

    #[cfg(feature = "alloc")]
    let w: Iart<i32> = Iart::new_err(MyError, "msg");
    #[cfg(not(feature = "alloc"))]
    let w: Iart<i32> = Iart::new_err(&MyError, "msg");

    let err = w.unwrap_err();

    assert_eq!(err.detail.desc.unwrap(), "msg");
}

#[test]
#[cfg(feature = "for-nightly-allocator-api-support")]
fn test_allocator_ok() {
    let _guard = TEST_LOG_LOCK.lock();

    let w: Iart<i32> = Iart::new_ok_in(42, Global);
    assert!(w.is_ok().unwrap());
    assert_eq!(w.unwrap(), 42);
}

#[test]
fn test_no_drop_after_unwrap() {
    let _guard = TEST_LOG_LOCK.lock();

    set_handler(test_logger);
    clear();

    {
        let w: Iart<i32> = Iart::new_ok(5);
        let _ = w.unwrap();
    }

    let history = LOG_HISTORY.lock();

    #[cfg(feature = "alloc")]
    assert!(!history.contains(&"dropped"));
    #[cfg(not(feature = "alloc"))]
    assert!(!history.contains(&Some("dropped")));
}

#[test]
#[cfg(all(
    feature = "check-unused-result",
    feature = "std",
    not(feature = "danger-allow-panic-if-unused")
))]
fn test_drop_without_handling_if_panic_raised() {
    let _guard = TEST_LOG_LOCK.lock();

    set_handler(test_logger);
    {
        clear();
    }

    let _ = std::panic::catch_unwind(|| {
        #[cfg(feature = "alloc")]
        let _w: Iart<i32> = Iart::new_err(MyError, "panic test");
        #[cfg(not(feature = "alloc"))]
        let _w: Iart<i32> = Iart::new_err(&MyError, "panic test");

        panic!("intentional panic");
    });

    let history = LOG_HISTORY.lock();

    #[cfg(feature = "alloc")]
    assert!(
        !history.contains(&"dropped"),
        "Should not log 'dropped' event during panic unwinding to avoid double panic."
    );

    #[cfg(not(feature = "alloc"))]
    assert!(
        !history.contains(&Some("dropped")),
        "Should not log 'dropped' event during panic unwinding to avoid double panic."
    );
}

#[test]
fn test_downcast_to_original_error() {
    let _guard = TEST_LOG_LOCK.lock();

    let w = {
        #[cfg(feature = "alloc")]
        let w: Iart<i32> = Iart::new_err(crate::tests::MyError, "TEST");
        #[cfg(not(feature = "alloc"))]
        let w: Iart<i32> = Iart::new_err(&MyError, "TEST");

        w
    };

    let detail = unsafe { w.try_downcast::<MyError>().expect("failed to downcast.") };

    #[cfg(feature = "alloc")]
    assert_eq!(detail.detail.desc.unwrap().as_ref(), "TEST");
    #[cfg(not(feature = "alloc"))]
    assert_eq!(detail.detail.desc.unwrap(), "TEST");
}

#[test]
fn new_version_ok_and_err() {
    let _guard = TEST_LOG_LOCK.lock();

    let mut w = {
        #[cfg(feature = "alloc")]
        let w: Iart<i32> = Iart::new_err(MyError, "TEST");
        #[cfg(not(feature = "alloc"))]
        let w: Iart<i32> = Iart::new_err(&MyError, "TEST");

        w
    };

    assert!(!w.handled);

    match w.ok() {
        Ok(_) => {
            panic!("Not ok, but returned Ok");
        }
        Err(w_moved) => {
            w = w_moved;
        }
    }

    assert!(!w.handled);

    match w.err() {
        Err(_) => {
            panic!("failed to match")
        }
        Ok(_) => {}
    }
}

#[test]
fn cast_from() {
    let _guard = TEST_LOG_LOCK.lock();

    #[cfg(feature = "alloc")]
    fn f() -> Result<u32, MyError> {
        Err(MyError)
    }

    #[cfg(feature = "alloc")]
    fn f2() -> Result<u32, MyError> {
        Ok(56)
    }

    #[cfg(not(feature = "alloc"))]
    fn f() -> Result<u32, &'static MyError> {
        Err(&MyError)
    }

    #[cfg(not(feature = "alloc"))]
    fn f2() -> Result<u32, &'static MyError> {
        Ok(56)
    }

    let res: Iart<u32> = f().into();
    assert!(res.is_err().unwrap());
    let _ = res.unwrap_err();

    let res: Iart<u32> = f2().into();
    assert_eq!(res.unwrap(), 56);
}

#[test]
fn test_error_item() {
    let _guard = TEST_LOG_LOCK.lock();

    #[cfg(feature = "alloc")]
    fn test() -> Iart<u32> {
        Iart::new_err(MyError, "error").with_item(5u32)
    }
    #[cfg(not(feature = "alloc"))]
    fn test() -> Iart<u32> {
        Iart::new_err(&MyError, "error").with_item(5u32)
    }

    let res = test();
    let ret = res.unwrap_err();
    assert_eq!(ret.item, Some(5));
}

#[test]
fn test_map_and_handled_trace() {
    let _guard = TEST_LOG_LOCK.lock();

    let res = Iart::<u32>::new_ok(100u32);
    assert!(!res.handled);

    let res2 = res.map(|x| x);

    assert!(!res2.handled);
    assert_eq!(res2.unwrap(), 100);
}

#[test]
fn to_result() {
    let _guard = TEST_LOG_LOCK.lock();

    #[cfg(feature = "alloc")]
    let res = Iart::<u32>::new_err(MyError, "msg");
    #[cfg(not(feature = "alloc"))]
    let res = Iart::<u32>::new_err(&MyError, "msg");

    let res2 = Iart::<u32>::new_ok(5u32);
    let _ = unsafe { res.to_result::<MyError>() }
        .unwrap()
        .error_data
        .unwrap_err();
    unsafe { res2.to_result::<MyError>() }
        .unwrap()
        .error_data
        .unwrap();
}

#[test]
#[cfg(feature = "enable-pending-tracker")]
fn tracker() {
    let _guard = TEST_LOG_LOCK.lock();

    assert_eq!(crate::is_found_pending_data(), false);

    let res: Iart<&str> = Iart::new_ok("hi!");

    assert_eq!(crate::is_found_pending_data(), true);

    let data = crate::get_current_tracking_data();

    let mut found = false;

    for i in data.iter() {
        let lock = i.lock();
        if let Some(_) = lock.as_ref() {
            found = true;
        }
    }

    assert_eq!(found, true);
    let _ = res.unwrap();
    assert_eq!(crate::is_found_pending_data(), false);

    found = false;

    for i in data.iter() {
        let lock = i.lock();
        if let Some(_) = lock.as_ref() {
            found = true;
        }
    }

    assert_eq!(found, false);
}
