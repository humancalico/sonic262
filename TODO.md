### TODO:

- Signal handling so that it gives the diagnostics even if the program is
  interrupted in the middle
- Make all the harness tests pass
- Use `evmap` as the concurrent hashmap. Currently we use `dashmap` as the
  concurrent hashmap which has a very easy to use API but `evmap` might be more
  suitable for our usecase since it is highly read-optimised and better in cases
  in which writes are less.
- Should give better error when test and include directories are not found.
```
Error:
   0: No such file or directory (os error 2)

Location:
   src/lib.rs:156
```

