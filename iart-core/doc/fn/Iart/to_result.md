Converts [`Iart`] to [`Result`].

# Errors

1. If `data` is None
2. If downcasting the error fails

# Safety

1. If the data type inside and the data held in the conversion function no longer match, a `UB` will occur during the
   conversion to [`core::any::Any`].
   Therefore, ensure that the consistency is correct.

> [!TIP]
> However, simply entering the wrong argument is not a problem.
> An error will occur, but a `UB` will not happen.
