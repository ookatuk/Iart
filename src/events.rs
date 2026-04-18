use core::fmt::Formatter;

#[non_exhaustive]
pub enum AutoRequestType {
    TryUsed,
    Expect,
    GetOk,
    GetErr,
    Unwrap,
    UnwrapErr,
}

#[non_exhaustive]
pub enum IartEvent<'a, 'b> {
    DroppedWithoutCheck,
    FunctionHook(AutoRequestType),
    TraceOverFlow,
    DisplayRequest(&'a mut Formatter<'b>),
    DebugRequest(&'a mut Formatter<'b>),
}
