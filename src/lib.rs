#![cfg_attr(feature = "strict", deny(warnings))]
//! Retrogress provides thread-safe progress bars with a simple and limited API that can be
//! passed between modules and functions.
//!
//! This crate provides progress bars that can be passed around, borrowed, or moved between
//! threads, with implementations for both single-threaded and multi-threaded scenarios.
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

pub mod error;
pub mod parallel;
pub mod progress;
pub mod render;
pub mod sync;

pub use error::Error;
pub use parallel::Parallel;
pub use progress::Progress;
pub use progress::ProgressBar;
pub use sync::Sync;
