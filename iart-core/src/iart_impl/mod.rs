#![doc = include_str!("../../doc/modules/iart_impl.md")]

#[cfg(feature = "allow-backtrace-logging")]
mod allow_backtrace_logging;
mod impls;
mod nightly_support;

mod drop_check;
