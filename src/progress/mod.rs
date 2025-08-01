use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

/// A reference to a specific progress bar.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    #[derive(Default)]
    struct MockProgress {
        appended: Arc<Mutex<Vec<(Ref, String)>>>,
        failed_refs: Arc<Mutex<Vec<Ref>>>,
        succeeded_refs: Arc<Mutex<Vec<Ref>>>,
        hidden_refs: Arc<Mutex<Vec<Ref>>>,
        shown_refs: Arc<Mutex<Vec<Ref>>>,
        println_calls: Arc<Mutex<Vec<(Ref, String)>>>,
        set_message_calls: Arc<Mutex<Vec<(Ref, String)>>>,
    }

    impl MockProgress {
        fn new() -> Self {
            Self::default()
        }
    }

    impl Progress for MockProgress {
        fn append(&mut self, msg: &str) -> Ref {
            let reference = Ref::new();
            self.appended
                .lock()
                .unwrap()
                .push((reference, msg.to_string()));
            reference
        }

        fn failed(&mut self, reference: Ref) {
            self.failed_refs.lock().unwrap().push(reference);
        }

        fn hide(&mut self, reference: Ref) {
            self.hidden_refs.lock().unwrap().push(reference);
        }

        fn println(&mut self, reference: Ref, msg: &str) {
            self.println_calls
                .lock()
                .unwrap()
                .push((reference, msg.to_string()));
        }

        fn set_message(&mut self, reference: Ref, msg: String) {
            self.set_message_calls
                .lock()
                .unwrap()
                .push((reference, msg));
        }

        fn show(&mut self, reference: Ref) {
            self.shown_refs.lock().unwrap().push(reference);
        }

        fn succeeded(&mut self, reference: Ref) {
            self.succeeded_refs.lock().unwrap().push(reference);
        }
    }

    #[test]
    fn ref_new_generates_unique_ids() {
        let ref1 = Ref::new();
        let ref2 = Ref::new();
        let ref3 = Ref::new();

        assert_ne!(ref1, ref2);
        assert_ne!(ref2, ref3);
        assert_ne!(ref1, ref3);
    }

    #[test]
    fn ref_default_creates_new_ref() {
        let ref1 = Ref::default();
        let ref2 = Ref::default();

        assert_ne!(ref1, ref2);
    }

    #[test]
    fn ref_is_copy_and_clone() {
        let ref1 = Ref::new();
        let ref2 = ref1; // Copy
        let ref3 = ref1.clone(); // Clone

        assert_eq!(ref1, ref2);
        assert_eq!(ref1, ref3);
        assert_eq!(ref2, ref3);
    }

    #[test]
    fn ref_can_be_used_as_hash_key() {
        let mut map = HashMap::new();
        let ref1 = Ref::new();
        let ref2 = Ref::new();

        map.insert(ref1, "value1");
        map.insert(ref2, "value2");

        assert_eq!(map.get(&ref1), Some(&"value1"));
        assert_eq!(map.get(&ref2), Some(&"value2"));
    }

    #[test]
    fn progress_bar_new_wraps_implementation() {
        let mock = MockProgress::new();
        let _progress_bar = ProgressBar::new(Box::new(mock));
        // If this compiles and runs without panicking, the test passes
    }

    #[test]
    fn progress_bar_append_delegates_to_implementation() {
        let mock = MockProgress::new();
        let appended_calls = Arc::clone(&mock.appended);
        let mut progress_bar = ProgressBar::new(Box::new(mock));

        let ref1 = progress_bar.append("test message 1");
        let ref2 = progress_bar.append("test message 2");

        let calls = appended_calls.lock().unwrap();
        assert_eq!(
            *calls,
            vec![
                (ref1, "test message 1".to_string()),
                (ref2, "test message 2".to_string())
            ]
        );
    }

    #[test]
    fn progress_bar_failed_delegates_to_implementation() {
        let mock = MockProgress::new();
        let failed_calls = Arc::clone(&mock.failed_refs);
        let mut progress_bar = ProgressBar::new(Box::new(mock));

        let reference = Ref::new();
        progress_bar.failed(reference);

        let calls = failed_calls.lock().unwrap();
        assert_eq!(*calls, vec![reference]);
    }

    #[test]
    fn progress_bar_succeeded_delegates_to_implementation() {
        let mock = MockProgress::new();
        let succeeded_calls = Arc::clone(&mock.succeeded_refs);
        let mut progress_bar = ProgressBar::new(Box::new(mock));

        let reference = Ref::new();
        progress_bar.succeeded(reference);

        let calls = succeeded_calls.lock().unwrap();
        assert_eq!(*calls, vec![reference]);
    }

    #[test]
    fn progress_bar_hide_delegates_to_implementation() {
        let mock = MockProgress::new();
        let hidden_calls = Arc::clone(&mock.hidden_refs);
        let mut progress_bar = ProgressBar::new(Box::new(mock));

        let reference = Ref::new();
        progress_bar.hide(reference);

        let calls = hidden_calls.lock().unwrap();
        assert_eq!(*calls, vec![reference]);
    }

    #[test]
    fn progress_bar_show_delegates_to_implementation() {
        let mock = MockProgress::new();
        let shown_calls = Arc::clone(&mock.shown_refs);
        let mut progress_bar = ProgressBar::new(Box::new(mock));

        let reference = Ref::new();
        progress_bar.show(reference);

        let calls = shown_calls.lock().unwrap();
        assert_eq!(*calls, vec![reference]);
    }

    #[test]
    fn progress_bar_println_delegates_to_implementation() {
        let mock = MockProgress::new();
        let println_calls = Arc::clone(&mock.println_calls);
        let mut progress_bar = ProgressBar::new(Box::new(mock));

        let reference = Ref::new();
        progress_bar.println(reference, "test output");

        let calls = println_calls.lock().unwrap();
        assert_eq!(*calls, vec![(reference, "test output".to_string())]);
    }

    #[test]
    fn progress_bar_set_message_delegates_to_implementation() {
        let mock = MockProgress::new();
        let set_message_calls = Arc::clone(&mock.set_message_calls);
        let mut progress_bar = ProgressBar::new(Box::new(mock));

        let reference = Ref::new();
        progress_bar.set_message(reference, "updated message".to_string());

        let calls = set_message_calls.lock().unwrap();
        assert_eq!(*calls, vec![(reference, "updated message".to_string())]);
    }

    #[test]
    fn progress_bar_is_send_and_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}

        assert_send::<ProgressBar>();
        assert_sync::<ProgressBar>();
    }

    #[test]
    fn progress_bar_clone() {
        let mock = MockProgress::new();
        let progress_bar = ProgressBar::new(Box::new(mock));
        let cloned_progress_bar = progress_bar.clone();

        drop(progress_bar);
        drop(cloned_progress_bar);
    }
}
