A structure that holds functions for the mutual conversion between [`core::any::Any`] and `dyn IartErr`.

# Safety

Since a type mismatch results in undefined behavior,

you must regenerate the error whenever changing the error type.