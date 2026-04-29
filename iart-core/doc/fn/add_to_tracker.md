Adding to the tracker.

# Locking
Uses `try_lock` only.

# Complexity
Worst: O(N) (N = RESULT_TRACK_MAX)
Best: O(1)
Avg: ~O(1 / (1 - α))  (α = load factor)

Note:
The average case assumes the scan start position is distributed
(e.g., rotated or randomized) and slot usage is relatively uniform.

# Why not `find()`?
`find()` performs a linear scan from a fixed position.
This implementation distributes the starting point, reducing contention
and improving average performance.

# Why not a free list?
A free list requires synchronization for both reads and writes.
Even lock-free approaches rely on atomic operations (e.g., CAS),
which can introduce overhead.

# Optimization
Enabling `enable-pending-tracker-tracking-count` helps avoid full scans
when full, but the related functions introduce SeqCst atomic overhead.