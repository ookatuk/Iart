**Set global handler.

1. If `true` is returned, the handler registration was successful.
2. If `false` is returned, someone has already registered.

# Warning
If you are using it in a library, never use it for anything other than `#[test]`.

Apps that use that library will not be able to register handlers.

# Note

1. Multi thread is supported.
2. This is not `RwLock`, This is Lock-Free(`AtomicPtr`).
3. If a handler is already set via other means, this function prevents double-registration via the `Once` guard.