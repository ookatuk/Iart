This macro behaves the same as `(iart)?` with `for-nightly-try-support` enabled.

In other words, it can be used in the same way as `?` and behaves the same way.

# Side Effects

1. The call location is recorded.
2. The handler is notified.

# Panics

1. Panic occurs if `data` is [`Option::None`].
