Attempts to downcast the error to `T`.

* If it fails, the original `self` is returned.

# Side Effects

1. If [`Iart::is_ok`] returns `false`, the following occur even if the downcast fails:
    - The handler is notified of [`TryDownCastUsed`].
    - The caller's location is recorded in the trace.
2. If the downcast fails after the above conditions are met:
    - The handler is notified of [`TryDownCastFail`].

# Errors

Returns `Err(self)` if:

- The result is successful ([`Iart::is_ok`] returns `true`).
- The internal `data` has already been consumed.
- The error cannot be downcast to type `T`.

# Safety

1. If the data type inside and the data held in the conversion function no longer match, a `UB` will occur during the
   conversion to [`core::any::Any`].
   Therefore, ensure that the consistency is correct.

> [!TIP]
> However, simply entering the wrong argument is not a problem.
> An error will occur, but a `UB` will not happen.
> A `UB` only occurs when there is an inconsistency between the internal data and the conversion function.
