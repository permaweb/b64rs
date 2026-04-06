# b64rs

Small Base64 helpers for Erlang, backed by Rust.

## What it does

- `b64rs:encode/1` turns binary data into URL-safe Base64 without padding.
- `b64rs:decode/1` turns it back into the original binary.

## Run tests

```sh
rebar3 eunit
```

## Run benchmarks

```sh
cargo bench --manifest-path native/b64rs/Cargo.toml --bench codec
```
