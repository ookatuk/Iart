#![cfg_attr(feature = "for-nightly-allocator-api-support", feature(allocator_api))]

extern crate alloc;

use iart_core::events::{AutoRequestType, IartEvent};
use iart_core::{Iart, IartHandleDetails, set_handler};
use iart_macros::{IartErr, iart_try};
use std::fmt::{Display, Formatter};

#[allow(unexpected_cfgs)]
#[derive(Debug, Clone, IartErr)]
struct MyErr {
    #[allow(unused)]
    data: &'static str,
}

impl Display for MyErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt("MyErr", f)
    }
}

impl core::error::Error for MyErr {}

fn add(x: i32, y: i32) -> Iart<i32> {
    Iart::new_ok(x + y)
}

fn error_raise() -> Iart<i32> {
    #[cfg(not(feature = "alloc"))]
    let data = &MyErr { data: "hi!" };
    #[cfg(feature = "alloc")]
    let data = MyErr { data: "hi!" };

    Iart::new_err(data, "example")
}

fn use_try() -> Iart<i32> {
    #[allow(unexpected_cfgs)]
    let _ = iart_try!(error_raise());

    unreachable!();
}

fn handler(event: IartEvent, iart: IartHandleDetails) -> core::fmt::Result {
    match event {
        IartEvent::DroppedWithoutCheck => {
            println!("non-checking dropped detected!");
            println!("---");

            println!("description: {:?}", iart.detail.unwrap().desc);
            println!("error: {:?}", unsafe {
                iart.detail.unwrap().clone().try_cast_err::<MyErr>()
            });
            println!("error?: {:?}", iart.is_err);
            println!("traces: {:?}", iart.log);

            println!("---");
        }
        IartEvent::FunctionHook(request) => match request {
            AutoRequestType::TryUsed => {
                if iart.is_err.unwrap() {
                    println!("try error!");
                }
            }
            _ => {}
        },
        _ => {}
    }
    Ok(())
}

fn main() {
    set_handler(handler);

    let res = add(5, 5);
    println!("5 + 5 = {}", res.unwrap());

    println!("---");
    let _ = error_raise();
    println!("---");

    println!("Check your logs/console for unused result warning!");

    println!("-- test2 ---");
    let _ = use_try();
    println!("---");
}
