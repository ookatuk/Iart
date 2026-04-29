This returns a reference to where the currently tracked data is stored.

1. If the [`Option`] in the `Mutex` within the slice is [`Some`], then it is being tracked.

2. If it is [`Option::None`], there is no tracked data here.

# Notes
Note that the method for adding new data to this is special, so just because the first entry is None does not mean it is not being tracked.