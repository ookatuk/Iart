This function retrieves a set of functions that perform mutual conversions between specific types that result in an
`IartErr` error internally and `core::any::Any`.

The functions included will cause a UB error under certain conditions (e.g., type mismatch).

# Safety

This is a macro function.
Running it in a way that disrupts the flow may result in a `panic!` or an error.
Also, in the worst-case scenario, `UB` may occur.