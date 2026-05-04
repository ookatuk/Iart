It behaves similarly to [`Result::map`].
If you have an `error-can-have-item`, it is recommended to use [`Iart::map_err_item`].

# Side Effects

1. The handler is notified.
2. A new one is created and the original [`Iart`] is deleted.
3. Almost all variables in the original [`Iart`] will have [`Option::take`] executed.

# Notes

* The newly created [`Iart`] is not a handled determination.
