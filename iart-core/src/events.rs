#![doc = include_str!("../doc/modules/events.md")]

use core::fmt::Formatter;

#[non_exhaustive]
#[doc = include_str!("../doc/structs/AutoRequestType.md")]
#[derive(Debug, PartialEq, Clone)]
pub enum AutoRequestType {
    TryUsed,
    Expect,
    GetOk,
    GetErr,
    Unwrap,
    UnwrapErr,
    TryDownCastFail,
    TryDownCastUsed,
    ToResultUsed,
    ToResultFail,
    Map,
    UnwrapUnchecked,
}

#[non_exhaustive]
#[doc = include_str!("../doc/structs/FailType.md")]
#[derive(Debug, PartialEq, Clone)]
pub enum FailType {
    #[doc = include_str!("../doc/enum/FailType/FailToGetTraceDatabaseSlot.md")]
    FailToGetTraceDatabaseSlot,
    #[doc = include_str!("../doc/enum/FailType/FailToGetTrackerSlot.md")]
    FailToGetTrackerSlot,
}

#[non_exhaustive]
#[doc = include_str!("../doc/structs/IartEvent.md")]
pub enum IartEvent<'a, 'b> {
    #[doc = include_str!("../doc/enum/IartEvent/DroppedWithoutCheck.md")]
    DroppedWithoutCheck,
    #[doc = include_str!("../doc/enum/IartEvent/FunctionHook.md")]
    FunctionHook(AutoRequestType),
    #[doc = include_str!("../doc/enum/IartEvent/CreationDegraded.md")]
    CreationDegraded(FailType),
    #[doc = include_str!("../doc/enum/IartEvent/TraceOverFlow.md")]
    TraceOverFlow,
    #[doc = include_str!("../doc/enum/IartEvent/FmtRequest.md")]
    DisplayRequest(&'a mut Formatter<'b>),
    #[doc = include_str!("../doc/enum/IartEvent/FmtRequest.md")]
    DebugRequest(&'a mut Formatter<'b>),
}
