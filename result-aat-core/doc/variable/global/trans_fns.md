A set of functions that can convert dyn [`IartErr`] to dyn [`core::any::Any`],
or vice versa.

# Safety

Since a type mismatch results in undefined behavior,

you must regenerate the error whenever changing the error type.