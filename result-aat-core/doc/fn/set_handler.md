Set global handler.

# Note

1. Multi thread is supported.
2. This is not `RwLock`, This is Lock-Free.
3. If you register two handlers, the first handler you registered will no longer be handled.