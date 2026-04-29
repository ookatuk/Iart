A closure that changes `dyn IartErr` to [`core::any::Any`].
That's all!
# Safety
1. This closure has a fixed set of types it can convert to, so using it carelessly will result in `UB`.