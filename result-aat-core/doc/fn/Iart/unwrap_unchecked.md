`unwrap_unchecked` is executed assuming that `data` is [`Some`] and [`Result::Ok`].

# Side Effects

1. [`Option::take`] is executed on the data.
2. The handler is notified.
3. The execution location is recorded.

# Safety

1. If `data` is not [`Some`], `UB` occurs via [`Option::take`]->[`Option::unwrap_unchecked`].
2. If `data` is not [`Ok`], `UB` occurs in [`Result::unwrap_unchecked`].
