//! # Iart: **I**s **A**dvanced **R**esult **T**race
//!
//! a structure inspired by [`Result`], designed for `std` and `no-std`.
//! supporting event-driven handling and dynamic tracing.
//!
//! ## Features
//!
//! 1. **Event Notification**: Automatically notifies handlers when error-handling methods are executed.
//! 2. **`no-std` Tracing**: Lightweight and simple execution tracing that works in embedded environments.
//! 3. **Usage Validation**: Issues warnings if the result is not handled properly (goes beyond simple `is_err` checks).
//! 4. **works on `stable` Rust**: See the `Nightly build only?` section.
//!
//! ## Nightly build only?
//! **No!**
//! I've ensured it **works perfectly**(with `cargo hack test`) with stable builds as well!
//!
//! While some syntax is limited, usability is only **slightly affected.**
//! (For example, you'll use `iart_try!` instead of the `?` operator.)
//!
//! **Crucially, almost all core features are NOT restricted!**
//! (Everything except those explicitly marked with `for-nightly-` feature flags.)
//!
//! Please give it a try, even on your stable toolchain!
//!
//!
//!
//! ## Examples(It works in stable)
//!
//! ```
//! use iart::prelude::Iart;
//! use iart::prelude::DummyErr;
//! use iart::iart_try;
//! use core::panic::Location;
//!
//! use std::collections::VecDeque;
//! // or alloc::collections::VecDeque
//!
//! fn main() {
//!     // 1. Success
//!     let res = Iart::Ok("hi");
//!
//!     // 2. Errors with diagnostic messages
//!     // Use `Err` for static messages or `Err_string` for dynamic ones (like `format!`).
//!     let res_err1: Iart<i32> = Iart::Err(DummyErr{}, "Static error message"); // **NOT Enum! This is function!**
//!     let res_err2: Iart<u32> = Iart::Err_string(DummyErr{}, format!("Dynamic error: {}", 404));
//!
//!     // or Iart<u32> = Iart::Err_item(DummyErr{}, "test", 56); // `error-can-have-item`
//!
//!     let res_err1 = res_err1.ok().err().unwrap(); // ok function is if result is ok, return `i32`, if not ok, return self
//!
//!     let result: Iart<u32> = core::result::Result::Err(DummyErr{}).into(); // Can This!(if IartErr is included in struct, From impl supported)
//!
//!     fn test() -> Iart<u32> {
//!         let result: Iart<u32> = Iart::Ok(5);
//!
//!         // in nightly build,
//!         // result? is supported(`for-nightly-try-support` feature)
//!
//!         let res: u32 = iart_try!(result); // for stable build
//!         // or `iart_open_no_log!` (can use in all build)
//!         Iart::Ok(res)
//!     }
//!
//!     let res = test().unwrap();
//!
//!     assert_eq!(res, 5);
//!
//!     let res: Result<(Result<u32, Box<DummyErr>>, Option<VecDeque<&'static Location<'static>>>), Iart<u32>> = test().to_result(); // can this!
//!
//!     // This can be done even under normal circumstances.
//!     res_err1.for_each_log(|log: &'static Location<'static>| -> bool {
//!         println!("{:?}", log);
//!         true
//!     });
//!
//!     // 3. Automatic Warning on Drop
//!     // If an error is dropped without being handled, iart automatically notifies the handler.
//!     drop(res_err1); // Triggers a warning to the handler
//!
//!     // 4. Proper Handling
//!     // Methods like `unwrap_err()` mark the instance as "handled," suppressing the drop warning.
//!     match res_err2.unwrap_err() {
//!         (detail, _) => {
//!             // `detail.desc` provides access to the stored diagnostic message.
//!             println!("Handled error: {:?}", detail.desc);
//!         }
//!     }
//! }
//! ```
//!
//! ## Wait? can I set max traces?
//! A: **YES!**
//! If you need set max, Please set env `IART_TRACE_MAX=(number)`(I'm using env macro(not func))
//! If you need select Delete type,
//! set `IART_TRACE_TYPE=good/last/first`
//! * `good` - A system that caused the error is not deleted; instead, the old version that was used as an intermediary is deleted and a new version is installed.
//! * `first` - When a new one arrives, if the limit is reached, it will skip over the old one instead of overwriting it.
//! * `last` - A system where new data overwrites old data.
//!
//! ## Runtime costs?
//! **Did you think the runtime cost was too high**?
//! Don't worry.
//! Most of the features of this structure (error tracing(`allow-backtrace-logging`), detection of unused errors(`check-unused-result`)) can be toggled using features.
//! There are also features not mentioned here, so please take a look.
//!
//! ### Regarding performance
//! It's at least slower than result, but
//! you might see improvement by turning off all features (which almost completely eliminates the cost of dropping) or by enabling `for-nightly-likely-optimization` when release build!
//!
//! ## Sometimes you want to return the same struct on failure as you do on success, for a specific reason, right?
//! The `error-can-have-item` feature comes in handy!
//!
//! Of course, we welcome suggestions!
//!
//! ## Advanced Examples(works in stable)
//!```
//! use iart::prelude::Iart;
//! use iart::prelude::DummyErr;
//! use iart::prelude::events::IartEvent;
//! use iart::prelude::events::AutoRequestType;
//! use iart::prelude::IartHandleDetails;
//! use iart::prelude::set_handler;
//! use iart::IartErr;
//! use core::fmt;
//!
//! #[derive(IartErr, Debug, Clone)] // `IartErr` required `Send`, `Sync`, `Clone` and `Debug`, Display, (If you use generic_member_access, need [`core::error::Error`])
//! struct Error{
//!
//! }
//!
//! impl core::fmt::Display for Error { // Important parts such as `expect` are not passed to the handler, so you need to specify them.
//!     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//!         write!(f, "{:?}", self)
//!      }
//! }
//!
//! // During panics, including with `no-std` builds, the allocator is not used automatically. (Probably)
//! fn handler(event: IartEvent, iart: IartHandleDetails) -> core::fmt::Result { //
//!     match event {
//!         IartEvent::DebugRequest(fmt) => {
//!             write!(fmt, "debug fmt")?;
//!         },
//!         IartEvent::DroppedWithoutCheck => {// When using `std`, this method is not called during a panic.
//!             println!("success detect!");
//!         },
//!         IartEvent::FunctionHook(hook_type) => {
//!             match hook_type {
//!                 AutoRequestType::Unwrap => {
//!                     // Retrieve the last recorded location from the audit log
//!                     if let Some(location) = iart.log.and_then(|l| l.back()) {
//!                         println!("Audit: unwrap detected at {}:{}!", location.file(), location.line());
//!                     }
//!                 },
//!                 _ => {}
//!             }
//!         }
//!         _ => {}
//!     }
//!     Ok(())
//! }
//!
//! fn main() {
//!     set_handler(handler); // It will be stored atomically.
//!     // and optional
//!     // if you need default handler, enable `enable-default-handler`
//!     // ...
//! }
//! ```
//!
//! **Note:** This structure requires `alloc`.
//!
//! If you need to use a specific allocator instead of the global allocator, we recommend enabling the `for-nightly-allocator-api-support` feature.
//! Note: This feature is currently a **TODO**. Both the functionality and implementation are unstable.
//! To use it, you must manually uncomment the relevant lines in Cargo.toml.
//!
//! ## Want to Panic?
//! Use now `danger-allow-panic-if-unused`!
//!
//! ## Is this using AI?
//! A:
//! > Yes,
//! > But we also conduct thorough execution tests(`cargo hack test` and other) and visual checks,
//! >  and I'll say that I wrote 80% of it **myself**.
//!
//! ## Features
//! `std`
//! > This enables the standard format of the dependent crate.
//! >
//! > It is also used as a criterion for determining whether something is standard,
//! > It is also used for other purposes, such as enabling/disabling panic checks during `std` builds.
//!
//! `for-nightly-likely-optimization` - (nightly) Unlikely and cold_path are placed in areas where abnormal processing occurs, and the placement of paths that are normally accessed is optimized.
//! `for-nightly-try-support` - (nightly) Supports `try_api_v2` and enables `Iart::Err()?`.
//! `for-nightly-error-generic-member-access` - (nightly)
//!
//! `no-trace-dedup` - If there are consecutive traces from the same location, the function to not add will be **disabled**.
//! `allow-backtrace-logging`
//! > When an error is logged,
//! > The source code location is recorded from the time the structure is created until,
//! > A method that destroys the structure is executed.
//! > This uses `Location::caller()`.
//! `allow-backtrace-logging-with-ok` - `allow-backtrace-logging` is also applied when the result is OK.
//! `check-unused-result` - This feature reports to a handler if an item is dropped and the error details are not handled clearly.
//! `check-unused-result-with-ok` - This feature checks `check-unused-result` even when the result is OK.
//! `danger-allow-panic-if-unused` - If unused is detected, `panic!` will be executed after the handler has finished processing.
//! `error-can-have-item`
//! > `Err_item`, `Err_item_option`, etc., will be added.
//! > Please note that these will be passed in a special format.
//! > For example, in the case of the `err` method, it is passed as the second tuple.
//! `core_error-support`
//! > It cannot coexist with `std` Because,
//! > `core_error::Error` becomes `std::error::Error`
//! >
//! > Otherwise, `core_error::Error` will be implemented correctly.
//! `ignore-handler-err`
//! > Normally, when a handler returns an error,
//! > it either reports it to fmt or panics, but if this feature is enabled,
//! > it will either report it to fmt or ignore it.
//! `enable-default-handler`
//! > `std` is required.
//! >
//! > It's okay if this feature is invalid or if set_handler isn't called.
//! >
//!>  However, if unused is detected in the case of `std`, it simply calls `eprintln!`.
//!
//! ### Todo Features
//! `for-nightly-allocator-api-support` - (nightly) Enables `allocator-api` support. Usage is the same as `core`, using `new_in`, etc.

pub mod prelude {
    pub use iart_core::*;
}
pub use iart_macros::*;
