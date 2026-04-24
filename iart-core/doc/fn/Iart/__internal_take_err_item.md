Take the item in case of an error.

If the `error-can-have-item` feature is turned off, it will always return [`None`].

# Safety

This is a macro function.
Running it in a way that disrupts the flow may result in a `panic!` or an error.
Also, in the worst-case scenario, `UB` may occur.