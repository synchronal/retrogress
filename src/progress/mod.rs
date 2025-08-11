use console::Term;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

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

impl std::fmt::Display for Ref {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#Ref<{}>", self.0)
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
    /// Function to rerender the progress bar. This will be called on a
    /// regular interval.
    fn render(&mut self);
    /// Update the message shown for a progress bar.
    fn set_message(&mut self, reference: Ref, msg: String);
    /// Shows the given progress bar.
    fn show(&mut self, reference: Ref);
    /// Mark the given progress bar as succeeded.
    fn succeeded(&mut self, reference: Ref);
}

enum ProgressMessage {
    Tick,
    Shutdown,
}

pub struct ProgressBar {
    progress: Arc<Mutex<Box<dyn Progress>>>,
    renderer: Arc<Mutex<Option<JoinHandle<()>>>>,
    sender: Arc<Mutex<Sender<ProgressMessage>>>,
    ticker: Arc<Mutex<Option<JoinHandle<()>>>>,
    ref_count: Arc<AtomicUsize>,
}

unsafe impl std::marker::Send for ProgressBar {}
unsafe impl std::marker::Sync for ProgressBar {}

impl ProgressBar {
    pub fn new(bar: Box<dyn Progress>) -> Self {
        let _ = Term::stdout().hide_cursor();
        let _ = Term::stderr().hide_cursor();
        let (sender, receiver) = mpsc::channel();

        #[allow(clippy::arc_with_non_send_sync)]
        let progress = Self {
            progress: Arc::new(Mutex::new(bar)),
            renderer: Arc::new(Mutex::new(None)),
            sender: Arc::new(Mutex::new(sender.clone())),
            ticker: Arc::new(Mutex::new(None)),
            ref_count: Arc::new(AtomicUsize::new(1)),
        };

        let progress_clone = progress.clone();
        let renderer = thread::spawn(move || {
            Self::start_renderer(progress_clone, receiver);
        });

        let ticker_sender = sender.clone();
        let ticker = thread::spawn(move || loop {
            thread::sleep(Duration::from_millis(50));
            if ticker_sender.send(ProgressMessage::Tick).is_err() {
                break;
            }
        });

        {
            let mut render_handle = progress.renderer.lock().unwrap();
            *render_handle = Some(renderer);
        }
        {
            let mut ticker_handle = progress.ticker.lock().unwrap();
            *ticker_handle = Some(ticker);
        }

        progress
    }

    pub fn append(&mut self, msg: &str) -> Ref {
        self.progress.lock().unwrap().append(msg)
    }
    pub fn failed(&mut self, reference: Ref) {
        self.progress.lock().unwrap().failed(reference)
    }
    pub fn hide(&mut self, reference: Ref) {
        self.progress.lock().unwrap().hide(reference)
    }
    pub fn println(&mut self, reference: Ref, msg: &str) {
        self.progress.lock().unwrap().println(reference, msg)
    }
    pub fn set_message(&mut self, reference: Ref, msg: String) {
        self.progress.lock().unwrap().set_message(reference, msg)
    }
    pub fn show(&mut self, reference: Ref) {
        self.progress.lock().unwrap().show(reference)
    }
    pub fn succeeded(&mut self, reference: Ref) {
        self.progress.lock().unwrap().succeeded(reference)
    }

    fn start_renderer(progress: ProgressBar, receiver: Receiver<ProgressMessage>) {
        while let Ok(message) = receiver.recv() {
            match message {
                ProgressMessage::Tick => progress.progress.lock().unwrap().render(),
                ProgressMessage::Shutdown => {
                    break;
                }
            }
        }
    }
}

impl Clone for ProgressBar {
    fn clone(&self) -> Self {
        self.ref_count.fetch_add(1, Ordering::SeqCst);
        Self {
            progress: Arc::clone(&self.progress),
            renderer: Arc::clone(&self.renderer),
            sender: Arc::clone(&self.sender),
            ticker: Arc::clone(&self.ticker),
            ref_count: Arc::clone(&self.ref_count),
        }
    }
}

impl Drop for ProgressBar {
    fn drop(&mut self) {
        let prev_count = self.ref_count.fetch_sub(1, Ordering::SeqCst);
        let _ = self.sender.lock().unwrap().send(ProgressMessage::Shutdown);

        if prev_count == 1 {
            let mut join_handle = self.ticker.lock().unwrap();
            if let Some(ticker) = join_handle.take() {
                let _ = ticker.join();
            }

            let mut join_handle = self.renderer.lock().unwrap();
            if let Some(renderer) = join_handle.take() {
                let _ = renderer.join();
            }
        }
        let _ = Term::stdout().show_cursor();
        let _ = Term::stderr().show_cursor();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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

        fn render(&mut self) {}

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
        let ref3 = ref1; // Clone

        assert_eq!(ref1, ref2);
        assert_eq!(ref1, ref3);
        assert_eq!(ref2, ref3);
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
