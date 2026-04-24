This is essentially the same as [`Result::expect`].

# Side Effects

1. The contents of `data` are moved.
2. The handler is notified.
3. The execution location is recorded.

# Panics

1. If `data` is [`Option::None`]
2. If the contents of `data` are [`Result::Err`]
