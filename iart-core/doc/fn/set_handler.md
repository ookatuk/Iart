Set global handler.

1. If `true` is returned, the handler registration was successful.
2. If `false` is returned, someone has already registered.

# Warning
1. If you are developing a library,
2. I recommend not registering handlers unless it is for testing.(The app will no longer be able to handle it.)

# Note

1. Multi thread is supported.
2. This is not `RwLock`, This is Lock-Free.
3. If you register two handlers, the first handler you registered will no longer be handled.