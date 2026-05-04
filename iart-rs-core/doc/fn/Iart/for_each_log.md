The program iterates through the closure provided with the backtrace using `for_each`.

If the closure returns true, it exits without continuing.

If there are no errors, or if it succeeds and the program is configured not to retain traces on success, it does not
execute.