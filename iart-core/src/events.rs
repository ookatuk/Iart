#![doc = include_str!("../doc/modules/events.md")]

use core::fmt::Formatter;

#[non_exhaustive]
#[doc = include_str!("../doc/structs/AutoRequestType.md")]
pub enum AutoRequestType {
    TryUsed,
    Expect,
    GetOk,
    GetErr,
    UnwrapUsed,
    UnwrapErr,
    TryDownCastFail,
    TryDownCastUsed,
    ToResultUsed,
    ToResultFail,
    Map,
    UnwrapUnchecked,
}

#[non_exhaustive]
pub enum FailType {
    // TODO: DOC
    FailToGetTraceDatabaseSlot,
    FailToGetTrackerSlot,
}

#[non_exhaustive]
#[doc = include_str!("../doc/structs/IartEvent.md")]
pub enum IartEvent<'a, 'b> {
    DroppedWithoutCheck,
    FunctionHook(AutoRequestType),
    CreationDegraded(FailType),
    TraceOverFlow,
    DisplayRequest(&'a mut Formatter<'b>),
    DebugRequest(&'a mut Formatter<'b>),
}
