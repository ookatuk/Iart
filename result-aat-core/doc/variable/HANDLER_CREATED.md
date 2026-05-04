A static variable that tracks whether a handle has been created.

# Why is this necessary?
It's necessary because if a default handler exists,
it will be mistakenly identified as having registered a handler.