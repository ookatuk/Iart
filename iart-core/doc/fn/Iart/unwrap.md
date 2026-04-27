This effectively performs [`Result::unwrap`].

# Side Effects

1. Handling occurs by the handler.
2. Data is [`Option::take`].
3. The call location is recorded.

# Panics
1. If `data` is [`None`] or [`Result::Err`]
2. If `item` is [`None`]