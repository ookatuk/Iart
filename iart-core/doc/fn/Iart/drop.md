This is executed when a structure is dropped.

# Side Effects

1.

> It is unprocessed and executed under certain conditions, such as the `check-unused-result` function, and the handler
> is called in blocking operations.

2.

> In a `std` environment, the handler is not called during a panic.

3.

> In a `non-std` environment, it is called even during a panic.

4.

> If the `danger-allow-panic-if-unused` feature is enabled, a `panic` will be called after the handler is invoked if the
> request is unhandled.