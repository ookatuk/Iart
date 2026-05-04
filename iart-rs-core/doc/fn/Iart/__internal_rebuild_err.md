[`Iart`] will be treated as an error and recreated without automation.

If the associated feature in the argument is invalid, it's okay to pass [`None`] there.

# Safety

This is a macro function.
Running it in a way that disrupts the flow may result in a `panic!` or an error.
Also, in the worst-case scenario, `UB` may occur.