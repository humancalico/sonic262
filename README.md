## sonic262

A blazing fast WIP harness for test262 written in Rust.

### Usage:

- Run all test262 tests

```sh
cargo build --release
./target/release/sonic262 --test-path ../test262/test/ --include-path ../test262/harness/
```

### TODO:

- Signal handling so that it gives the diagnostics even if the program is
  interrupted in the middle
- Make all the harness tests pass
- Use `evmap` as the concurrent hashmap. Currently we use `dashmap` as the
  concurrent hashmap which has a very easy to use API but `evmap` might be more
  suitable for our usecase since it is highly read-optimised and better in cases
  in which writes are less.
