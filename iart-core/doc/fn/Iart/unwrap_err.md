This effectively executes [`Result::unwrap_err`].
Note that if `error-can-have-item` is enabled, the second return value may contain `Item`.

# Side Effects

1. Handling occurs by the handler.
2. Data is [`Option::take`].
3. The call location is recorded.

# Panics

If `data` is [`None`] or [`Result::Ok`]
