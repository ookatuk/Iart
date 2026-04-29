The tracking ID is taken using [`Option::take`].

If `enable-pending-tracker` is disabled, it always returns [`Option::None`].

# Safety
This is a macro function.
Running it in a way that disrupts the flow may result in a `panic!` or an error.
Also, in the worst-case scenario, `UB` may occur.