Iart (Information and Result Trace)

Saves the results and unused flags.

You can enable other features by enabling the related features.

See the added methods for details.

# Try Infomation
If you want to return an error of a different type with `?`, use the `iart_try_without_item!` macro.


# Side Effects

When dropping,
if certain related functions are enabled,
they will be executed as **blocking** within the `drop` method.