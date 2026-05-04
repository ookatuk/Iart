This examines the error details.

Unlike [`Iart::err`], this method does not consume the result nor
trigger any side effects (e.g., logging or marking as handled).

Returns [`None`] if the result is [`Ok`], or if it has already been consumed.