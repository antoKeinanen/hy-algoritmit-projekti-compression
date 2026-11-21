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


## Web app

The `web` directory has a minimal React app for testing the algorithms in the browser. It uses the `wasm` crate, which exposes the algorithms to JavaScript.

With the devenv shell active:
```shell
cd web
pnpm install
pnpm dev
```

`pnpm dev` and `pnpm build` first build the wasm package into `wasm/pkg` with wasm-pack. The web app links to that package, so Rust changes show up after restarting `pnpm dev`. To rebuild only the wasm package, run `pnpm build:wasm`.
