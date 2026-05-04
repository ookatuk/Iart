A closure that changes [`core::any::Any`] to `dyn IartErr`.
That's all!
# Safety
1. This closure has a fixed set of types it can convert to, so using it carelessly will result in `UB`.