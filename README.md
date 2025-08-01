# Retrogress

Retrogress provides thread-safe progress bars with a simple and limited
API that can be passed between modules and functions.

This crate is designed for projects that need to share progress bars
between different sections of code. It provides implementations for both
single-threaded (Sync) and multi-threaded (Parallel) scenarios, allowing
progress bars to be passed around, borrowed, or moved between threads.

``` rust
cargo run --example sync
```
