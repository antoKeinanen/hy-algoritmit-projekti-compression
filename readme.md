# hy-algoritmit-projekti-compression

## Dependencies

- [Rust](https://rust-lang.org/tools/install/)

## Running tests

At the project root run:
```shell
cargo test
```

### Measuring test coverage

In Rust measuring test coverage requires some additional modules that can't be configured declaratively.

Setup with:
```shell
rustup component add llvm-tools-preview
cargo install cargo-llvm-cov
```

Then measure the coverage:
```shell
cargo llvm-cov
```


