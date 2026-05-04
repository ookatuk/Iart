global handler.

## How to use?

```ignore
let ptr = HANDLER.load(Ordering::Acquire);
if !ptr.is_null(){
    let logger: IartLogger = unsafe { core::mem::transmute(ptr) };
}
```

or

```ignore
let ptr = HANDLER.load(Ordering::Acquire);
if !ptr.is_null() {
    let logger: IartLogger<A> = unsafe { core::mem::transmute(ptr) };
}
```