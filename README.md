## sonic262

A blazing fast WIP harness for test262 written in Rust.

### Dependencies
- rust
- python(2) - for `rusty-v8`

### Usage:

- Run all test262 tests

```sh
cargo build --release
./target/release/sonic262 --test-path ../test262/test/ --include-path ../test262/harness/
```

