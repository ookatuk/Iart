# Iart: **I**s **A**dvanced **R**esult **T**race

[![Downloads](https://img.shields.io/crates/d/iart.svg)](https://crates.io/crates/iart)
[![Crates.io](https://img.shields.io/crates/v/iart.svg)](https://crates.io/crates/iart)
[![Docs.rs](https://docs.rs/iart/badge.svg)](https://docs.rs/iart)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

---

a structure inspired by [`Result`], designed for `std` and `no-std`.
supporting event-driven handling and dynamic tracing.

> Incidentally, I took a library I was already using personally, turned it into a crate, added features, and stabilized
> the specifications at the time of the initial release.
>
> If we were to change the specifications, it would be in version 1.0 or 2.0.

## Features

1. **Event Notification**: Automatically notifies handlers when error-handling methods are executed.
2. **`no-std` Tracing**: Lightweight and simple execution tracing that works in embedded environments.
3. **Usage Validation**: Issues warnings if the result is not handled properly (goes beyond simple `is_err` checks).
4. **works on `stable` Rust**: See the `Nightly build only?` section.

Perfect for projects expecting no-std support!

Except for the default handler,
functionality with no-std is not restricted!

## Examples(It works in stable)

```doctestinjectablerust
use iart::prelude::Iart;
use iart::prelude::DummyErr;
use iart::prelude::ErrorDetail;
use iart::iart_try;
use core::panic::Location;

use std::collections::VecDeque;
// or alloc::collections::VecDeque

fn main() {
    // 1. Success
    let res = Iart::Ok("hi");

    // 2. Errors with diagnostic messages
    // Use `Err` for static messages or `Err_string` for dynamic ones (like `format!`).
    let res_err1: Iart<i32> = Iart::Err(DummyErr {}, "Static error message"); // **NOT Enum! This is function!**
    let res_err2: Iart<u32> = Iart::Err_string(DummyErr {}, format!("Dynamic error: {}", 404));

    // or Iart<u32> = Iart::Err_item(DummyErr{}, "test", 56); // `error-can-have-item`

    let mut res_err1: Iart<i32> = res_err1.ok().err().unwrap(); // ok function is if result is ok, return `i32`, if not ok, return self

    let result: Iart<u32> = core::result::Result::Err(DummyErr {}).into(); // Can This!(if IartErr is included in struct, From impl supported)

    let _ = result.unwrap_err();
    let _ = res.unwrap();

    // if you need downcast, use [`Iart::try_downcast`]

    fn test() -> Iart<u32> { // Try is not supported by default, but if you want to use it...
        let result: Iart<u32> = Iart::Ok(5);

        // in nightly build,
        // result? is supported(`for-nightly-try-support` feature)

        let res: u32 = iart_try!(result); // for stable build
        // or `iart_open_no_log!` (can use in all build)
        Iart::Ok(res)
    }

    let res = test().unwrap();

    assert_eq!(res, 5);

    // Unless someone maliciously provides invalid values to the explicitly `unsafe` function [`iart_core::ErrorDetail::new`], UB will not occur in general use(and tests works).
    // Conversely, it is marked `unsafe` precisely because [`iart_core::ErrorDetail::new`] results in UB if given invalid values.
    //
    // Besides, why should we even need to account for manual tampering with [`iart_core::ErrorDetail::new`]?
    let res: Result<(Result<(), (DummyErr, Box<ErrorDetail>)>, Option<u32>, Option<VecDeque<&'static Location<'static>>>), Iart<u32>> = unsafe { test().to_result() }; // can this!

    // This can be done even under normal circumstances.
    res_err1.for_each_log(|log: &'static Location<'static>| -> bool {
        println!("{:?}", log);
        true
    });

    // 3. Automatic Warning on Drop
    // If an error is dropped without being handled, iart automatically notifies the handler.
    # unsafe { res_err1.__internal_mark_handled() };
    drop(res_err1); // Triggers a warning to the handler

    // 4. Proper Handling
    // Methods like `unwrap_err()` mark the instance as "handled," suppressing the drop warning.
    match res_err2.unwrap_err() {
        (detail, _) => {
            // `detail.desc` provides access to the stored diagnostic message.
            println!("Handled error: {:?}", detail.desc);
        }
    }
}
```
[see examples](./iart/examples)

## Nightly build only?

**No!**
I've ensured it **works perfectly**(with `cargo hack test`) with stable builds as well!

While some syntax is limited, usability is only **slightly affected.**
(For example, you'll use `iart_try!` instead of the `?` operator.)

**Crucially, almost all core features are NOT restricted!**
(Everything except those explicitly marked with `for-nightly-` feature flags.)

Please give it a try, even on your stable toolchain!

## Are you worried because it's too small?

I understand that if I were in your shoes, I'd be worried too.
However,
There are many features listed here as examples, but this is not an exhaustive list. (check now `features`)
since it's a library containing nearly 2500 lines of code, the size is reasonable, and the contents are manageable.
If you're worried, please do check it out.

## Wait? can I set max traces?

A: **YES!**
If you need set max, Please set env `IART_TRACE_MAX=(number)`(I'm using env macro(not func))
If you need select Delete type,
set `IART_TRACE_TYPE=good/last/first`

* `good` - A system that caused the error is not deleted; instead, the old version that was used as an intermediary is
  deleted and a new version is installed.
* `first` - When a new one arrives, if the limit is reached, it will skip over the old one instead of overwriting it.
* `last` - A system where new data overwrites old data.

## Runtime costs?

**Did you think the runtime cost was too high**?
Don't worry.
Most of the features of this structure (error tracing(`allow-backtrace-logging`), detection of unused errors(
`check-unused-result`)) can be toggled using features.
There are also features not mentioned here, so please take a look.

### Regarding performance

It's at least slower than result, but
you might see improvement by turning off all features (which almost completely eliminates the cost of dropping) or by
enabling `for-nightly-likely-optimization` when release build!

## Sometimes you want to return the same struct on failure as you do on success, for a specific reason, right?

The `error-can-have-item` feature comes in handy!

Of course, we welcome suggestions!

## Advanced Examples(works in stable)

```doctestinjectablerust
use iart::prelude::Iart;
use iart::prelude::DummyErr;
use iart::prelude::events::IartEvent;
use iart::prelude::events::AutoRequestType;
use iart::prelude::IartHandleDetails;
use iart::prelude::set_handler;
use iart::IartErr;
use core::fmt;

#[derive(
    IartErr,
    Debug,
    Clone
)] // `IartErr` required `Send`, `Sync`, `Clone` and `Debug`, Display, (If you use generic_member_access, need [`core::error::Error`])
struct Error {}

impl core::error::Error for Error {}

impl core::fmt::Display for Error { // Important parts such as `expect` are not passed to the handler, so you need to specify them.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

// During panics, including with `no-std` builds, the allocator is not used automatically. (Probably)
//If called for reasons other than handling `fmt`, the error will be ignored.
// `Debug/Display` event may be called for formatting during a panic.
fn handler(event: IartEvent, iart: IartHandleDetails) -> core::fmt::Result {
    match event {
        IartEvent::DebugRequest(fmt) => {
            write!(fmt, "debug fmt")?;
        }
        IartEvent::DroppedWithoutCheck => { // When using `std`, this method is not called during a panic.
            println!("success detect!");
        }
        IartEvent::FunctionHook(hook_type) => {
            match hook_type {
                AutoRequestType::Unwrap => {
                    // Retrieve the last recorded location from the audit log
                    if let Some(location) = iart.log.and_then(|l| l.back()) {
                        println!("Audit: unwrap detected at {}:{}!", location.file(), location.line());
                    }
                }
                _ => {}
            }
        }
        _ => {}
    }
    Ok(())
}

fn main() {
    set_handler(handler); // It will be stored atomically.
    // and optional
    // if you need default handler, enable `enable-default-handler`
    // ...
}
```

**Note:** This structure requires `alloc`.

If you need to use a specific allocator instead of the global allocator, we recommend enabling the
`for-nightly-allocator-api-support` feature.

## Want to Panic?

Use now `danger-allow-panic-if-unused`!

## Is this using AI?

A:
> Yes,
> But we also conduct thorough execution tests(`cargo hack test` and other) and visual checks,
> and I'll say that I wrote 95%(I also used AI only in the `tests.rs` section / document (of course, I wrote some of it
> myself as well).) of it **myself**.
>
> As I've said many times,
> 1. I check everything by reviewing it and running the code to make sure it's okay.
> 2. The implementation does not use AI.(Except for tests/doc)
> 3. Even when using AI, we perform visual checks and verify the processing.
> 4. Even if I were to use AI in my implementations in the future,
     > I would check the documentation for any unfamiliar functions before evaluating them.

## Expected Use Cases

1. To detect results that are suppressed or resolved with only `is_ok` at runtime.
2. To retain items even in case of errors.
3. To dynamically and statically retain text in addition to error cases.
4. To dynamically add errors from external sources.
5. To add a backtrace to [`Result`].
6. To obtain an environment-independent backtrace.
7. To obtain an easily understandable backtrace.
8. To use it intuitively without looking at the document, relying solely on predictive text.

## Features

`std`
> Enables std library support.
> This allows the use of standard types like [`std::error::Error`] and enables std-specific optimizations or panic
> behaviors.

`for-nightly-likely-optimization` - (nightly) Unlikely and cold_path are placed in areas where abnormal processing occurs, and the placement of paths that are normally accessed is optimized.

`for-nightly-try-support` - (nightly) Supports `try_api_v2` and enables `Iart::Err()?`.

`for-nightly-error-generic-member-access` - (nightly)

`no-trace-dedup` - If there are consecutive traces from the same location, the function to not add will be **disabled**.
`allow-backtrace-logging`
> When an error is logged,
> The source code location is recorded from the time the structure is created until,
> A method that destroys the structure is executed.
> This uses `Location::caller()`.

`allow-backtrace-logging-with-ok` - `allow-backtrace-logging` is also applied when the result is OK.

`check-unused-result` - This feature reports to a handler if an item is dropped and the error details are not handled clearly.

`check-unused-result-with-ok` - This feature checks `check-unused-result` even when the result is OK.

`danger-allow-panic-if-unused` - If unused is detected, `panic!` will be executed after the handler has finished processing.

`error-can-have-item`
> `Err_item`, `Err_item_option`, etc., will be added.
> Please note that these will be passed in a special format.
> For example, in the case of the `err` method, it is passed as the second tuple.

`core_error-support`
> It cannot coexist with `std` Because,
> `core_error::Error` becomes `std::error::Error`
>
> Otherwise, `core_error::Error` will be implemented correctly.

`enable-default-handler`
> `std` is required.
>
> It's okay if this feature is invalid or if set_handler isn't called.
>
> However, if unused is detected in the case of `std`, it simply calls `eprintln!`.
> 
`for-nightly-allocator-api-support` - (nightly) Enables `allocator-api` support. Usage is the same as `core`, using
`new_in`, etc.

`no-alloc`
> Supports static references instead of Box.
> Incompatible with for-nightly-allocator-api-support.

# How to install?
`cargo add iart`

## Mini Q&A

Q: I have something to worry about.

A:
> If it's about specifications, please use the Q&A section here or email me directly.
>
< If it's about practical use, try forking a small project and converting it into something you can do in your spare
time.
>
> Suggestions are very welcome!

Q: does Box::leak completely break it?(by reddit user)

A: 
> Unfortunately, that's correct. Since it's a system that triggers a sound based on 'Drop' detection, that's unfortunately how it should work.
>
> However, if we find an improvement for the nightly feature or something similar, we plan to gradually switch to that (as a feature)!
> 
> However, if it can't be detected at runtime, it's difficult to confirm whether it can be checked at compile time, so don't get your hopes up too much. 

Q: Will there be any disruptive changes?

A:
> There are no plans to do so for the time being.
>
> In the current plan, breaking changes only occur when you intentionally enable the feature flag

Q: It's so scary to use, I don't want to!(When converting to `any`, you might need to leak once and then convert it back
to a box using `into_raw`.)

A: Try the tests in all possible situations, or try different values until you feel confident.
> Even so, if you have a bad feeling about the conversion,
> you can still use it to some extent without it,
> and since it passed the tests,
> it should be working correctly.
> Also, it's only used internally!
>
> Don't worry!

Q: What about continuity?

A: we **are** take action if you report it.
> However, the order of responses may change,
> or only alternative solutions may be provided. (Even in such cases,
> we will review the methods as much as possible and strive to avoid changes that would compromise compatibility.)
>
> If I were to terminate the project,
> I would make sure to switch to read-only mode first.

Q: Is it compatible with `thiserror`?

A: It will work, and `thiserror` should fit in nicely.

Q: Is it okay if the code goes into a panic?

A: In a `std` environment where `unwind` is running,
> in the event of a panic, it simply terminates without doing anything.
> In the case of `no-std`, there is no detection method,
> but it should be fine because it probably doesn't use `alloc`.

Q: The warning when dropping is too severe.

A: I would appreciate it if you could either disable the feature or come up with some suggestions.

Q: I'm scared of macro dependencies.

A: You can either acquire the `regex` skill or try using the following code:

```ingnore
let res = match xxx.is_ok() {
    Ok(item) => {
    res
}
Err(err) => {
    err.send_log();
    return err;
}
```

Q: I'll use it as a library, but I don't want to release it externally.

A: Then let's use to_result.(A cast from dyn to the type specified in the argument is performed.)

Q: There's an `unsafe` method, but is it safe?

A:
> It's safe to use unless you perform some kind of black magic like crafting Iart via [`iart_core::ErrorDetail::new`].
>
> `UB` only occurs when two different types are involved.

Q: why `0.x.x`?

A:
> The bug detection rate differs between individual use and use by many people, so future fixes are possible.
>
> However, the fact remains that it was working with `no-std`, so the amount of fixes needed should be small compared to
> other projects.

Q: Can you even be trusted in the first place?

A:
> If you look at my GitHub contributions, you'll see that I've worked on multilingual support, bug fixes, and feature
> additions (and had them merged) in languages ​​other than Rust.
>
> If you're interested in Rust(and memory safety),
> you can see it in the code of my projects(pined). (My documentation might make you doubt my abilities, though.)
