This effectively executes [`Result::ok`].

# Side Effects

1. Handling occurs by the handler.
2. Data is [`Option::take`].
3. The call location is recorded.

# Errors

If `data` is [`None`] or [`Err`]
