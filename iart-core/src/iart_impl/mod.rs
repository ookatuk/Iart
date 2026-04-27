#![doc = include_str!("../../doc/modules/iart_impl.md")]

#[cfg(feature = "allow-backtrace-logging")]
mod allow_backtrace_logging;
mod impls;
mod nightly_support;

#[cfg(feature = "check-unused-result")]
mod check_unused_result;
