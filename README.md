# Rust bindings for the KING OF TIME API [![Crates.io](https://img.shields.io/crates/v/kingtime)](https://crates.io/crates/kingtime) [![docs.rs](https://img.shields.io/docsrs/kingtime)](https://docs.rs/kingtime/) ![main workflow](https://github.com/Idein/kingtime-rs/actions/workflows/main.yml/badge.svg)

## Example

Prints if you are at work or not at work.

```
$ cargo run --example tc -- status
```

Record the time you start working.

```
$ cargo run --example tc -- in
```

Record the time you finished working.

```
$ cargo run --example tc -- out
```

#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>
