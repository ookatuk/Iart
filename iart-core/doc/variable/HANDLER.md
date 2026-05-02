global handler.

## How to use?

```ignore
let ptr = HANDLER.load(Ordering::Acquire);
if !ptr.is_null(){
    let logger: IartHandler = unsafe { core::mem::transmute(ptr) };
}
```

or

```ignore
let ptr = HANDLER.load(Ordering::Acquire);
if !ptr.is_null() {
    let logger: IartHandler<A> = unsafe { core::mem::transmute(ptr) };
}
```