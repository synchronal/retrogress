use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

/// A reference to a specific progress bar.
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct Ref(usize);

impl Ref {
    /// May be used by implementers of new progress bar behaviors
    /// when a new progress bar is appended to the console.
    pub fn new() -> Self {
        static COUNTER: AtomicUsize = AtomicUsize::new(1);
        Self(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

impl Default for Ref {
    fn default() -> Self {
        Self::new()
    }
}

/// Progress is a trait that may be implemented to create new progress bar
/// behaviors. Its normal usage is as a `Box<dyn Progress>` that can be
/// put into a `ProgressBar`.
pub trait Progress {
    /// Append a new progress bar to the console. Returns a `usize` that
    /// serves as a reference to the progress bar in other functions.
    fn append(&mut self, msg: &str) -> Ref;
    /// Mark the given progress bar as failed.
    fn failed(&mut self, references: Ref);
    /// Hides the given progress bar.
    fn hide(&mut self, reference: Ref);
    /// Prints a line of text above a progress bar, without interrupted it.
    /// Helpful when capturing output from commands to show to users.
    fn println(&mut self, reference: Ref, msg: &str);
    /// Update the message shown for a progress bar.
    fn set_message(&mut self, reference: Ref, msg: String);
    /// Shows the given progress bar.
    fn show(&mut self, reference: Ref);
    /// Mark the given progress bar as succeeded.
    fn succeeded(&mut self, reference: Ref);
}

#[derive(Clone)]
pub struct ProgressBar(Arc<Mutex<Box<dyn Progress>>>);

unsafe impl std::marker::Send for ProgressBar {}
unsafe impl std::marker::Sync for ProgressBar {}

impl Progress for ProgressBar {
    fn append(&mut self, msg: &str) -> Ref {
        self.0.lock().unwrap().append(msg)
    }
    fn failed(&mut self, reference: Ref) {
        self.0.lock().unwrap().failed(reference)
    }
    fn hide(&mut self, reference: Ref) {
        self.0.lock().unwrap().hide(reference)
    }
    fn println(&mut self, reference: Ref, msg: &str) {
        self.0.lock().unwrap().println(reference, msg)
    }
    fn set_message(&mut self, reference: Ref, msg: String) {
        self.0.lock().unwrap().set_message(reference, msg)
    }
    fn show(&mut self, reference: Ref) {
        self.0.lock().unwrap().show(reference)
    }
    fn succeeded(&mut self, reference: Ref) {
        self.0.lock().unwrap().succeeded(reference)
    }
}

impl ProgressBar {
    pub fn new(bar: Box<dyn Progress>) -> Self {
        #[allow(clippy::arc_with_non_send_sync)]
        Self(Arc::new(Mutex::new(bar)))
    }
}
