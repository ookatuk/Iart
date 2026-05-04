If [`Iart`] is successful,
it contains an `Item`, and if [`Iart`] fails,
it contains [`ErrorDetail`] enclosed in a [`Box`].

Use `take` to retrieve it.
Also, note that it will become [`None`] when retrieved.