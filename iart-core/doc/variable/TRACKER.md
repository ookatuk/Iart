This is a static object that tracks unused [`Iart`] structs.

Basically, address integration by the Rust compiler will not affect consistency, so please rest assured.

# Notes
1. When address integration is performed by the Rust compiler, it may detect the [`Iart`] struct from within the library's internal context.

# Contents of `[&'static Location<'static>;2]`
* `[0]`: Location of occurrence
* `[1]`: Location where the last log was stored