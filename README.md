# Retrogress

Retrogress provides thread-safe progress bars with a simple and limited
API that can be passed between modules and functions.

This crate is designed for projects that need to share progress bars
between different sections of code. It provides implementations for both
single-threaded (Sync) and multi-threaded (Parallel) scenarios, allowing
progress bars to be passed around, borrowed, or moved between threads.

## Usage

``` rust
use retrogress::{ProgressBar, Sync};

let mut progress = ProgressBar::new(Sync::boxed());
let pb = progress.append("step 1");
progress.println(pb, "Write a line of text above the progress bar");
progress.succeeded(pb);
```

Use `Parallel` to run multible bars concurrently. A running bar shows
its latest 5 lines, cut to the terminal width. A failed bar prints its
latest 5 lines above the remaining bars, also cut to width. To print
more:

``` rust
use retrogress::{FailureOutput, Parallel, ProgressBar};

// Every line the bar printed, at full width.
let parallel = Parallel::new().failure_output(FailureOutput::Full);

// Or only its last 50 lines, at full width.
let parallel =
    Parallel::new().failure_output(FailureOutput::Limit { max_lines: 50 });

let mut progress = ProgressBar::new(Box::new(parallel));
```

## Examples

``` bash
cargo run --example sync
cargo run --example parallel
```
