#![doc = include_str!("../doc/modules/events.md")]

use core::fmt::Formatter;

#[non_exhaustive]
#[doc = include_str!("../doc/structs/AutoRequestType.md")]
pub enum AutoRequestType {
    TryUsed,
    Expect,
    GetOk,
    GetErr,
    Unwrap,
    UnwrapErr,
    TryDownCastFail,
    TryDownCastUsed,
    ToResult,
    ToResultFail,
    Map,
}

#[non_exhaustive]
#[doc = include_str!("../doc/structs/IartEvent.md")]
pub enum IartEvent<'a, 'b> {
    DroppedWithoutCheck,
    FunctionHook(AutoRequestType),
    TraceOverFlow,
    DisplayRequest(&'a mut Formatter<'b>),
    DebugRequest(&'a mut Formatter<'b>),
}
