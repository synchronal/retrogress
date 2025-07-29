#![cfg_attr(feature = "strict", deny(warnings))]
//! Retrogress is a wrapper around [indicatif](https://crates.io/crates/indicatif),
//! providing structs and traits that have a simple and limited API and that can be
//! passed between modules and functions.
//!
//! For advanced progress bar usage, in a project that does not need to share a progress
//! bar between different sections of code, one may be better served by directly using indicatif
//! (or any other of the many wonderful crates available). This crate can be used to
//! construct progress bars that can be passed around, borrowed, or moved between threads.
//!
//! ```rust
//! use retrogress::Progress;
//! use retrogress::{ProgressBar, Sync};
//!
//! let mut progress = ProgressBar::new(Sync::boxed());
//! let pb = progress.append("step 1");
//! progress.println(pb, "Write a line of text above the progress bar");
//! progress.println(pb, "Write another line");
//! progress.succeeded(pb);
//! ```

pub mod parallel;
pub mod progress;
pub mod sync;

pub use parallel::Parallel;
pub use progress::Progress;
pub use progress::ProgressBar;
pub use sync::Sync;
