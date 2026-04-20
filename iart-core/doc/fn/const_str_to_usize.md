&str to usize.

# Panics

* If used non-number.

# Example

```ignore
use crate::iart_core::utils::const_str_to_usize;

assert_eq!(const_str_to_usize("5685"), 5685)
```