# Retrogress

Retrogress is a wrapper around
[indicatif](https://crates.io/crates/indicatif), providing structs and
traits that have a simple and limited API and that can be passed between
modules and functions.

For advanced progress bar usage, in a project that does not need to
share a progress bar between different sections of code, one may be
better served by directly using indicatif (or any other of the many
wonderful crates available). This crate can be used to construct
progress bars that can be passed around, borrowed, or moved between
threads.

``` rust
cargo run --example sync
```
