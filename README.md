## sonic262
A blazing fast WIP harness for test262 written in Rust.

### Usage:

- Run all test262 tests

```
cargo build --release
./target/release/sonic262 --test-path ../test262/test/ --include-path ../test262/harness/
```

### TODO:
- Signal handling so that it gives the diagnostics even if the program is interrupted in the middle
- Make all the harness tests pass


