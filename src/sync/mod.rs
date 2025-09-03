use crate::progress::Ref;
use crate::render::Renderer;
use crate::Progress;
use console::Term;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// An implementation of `Progress` intended for usages when one
/// progress bar runs at a time.
///
/// ```rust
/// use retrogress::{ProgressBar, Sync};
/// let mut progress = ProgressBar::new(Sync::boxed());
/// ```
pub struct Sync {
    bars: Arc<Mutex<HashMap<Ref, Renderer>>>,
    current: Arc<Mutex<Option<Ref>>>,
    prompt: Option<String>,
}

impl Sync {
    pub fn new() -> Self {
        console::set_colors_enabled(true);
        console::set_colors_enabled_stderr(true);

        Self {
            bars: Arc::new(Mutex::new(HashMap::new())),
            current: Arc::new(Mutex::new(None)),
            prompt: None,
        }
    }

    pub fn boxed() -> Box<Self> {
        Box::new(Self::new())
    }
}

impl Default for Sync {
    fn default() -> Self {
        Self::new()
    }
}

impl Progress for Sync {
    fn append(&mut self, msg: &str) -> Ref {
        let pb = Renderer::new(msg.to_string());
        let reference = Ref::new();

        {
            let mut bars = self.bars.lock().unwrap();
            bars.insert(reference, pb);
        } // drop lock
        let mut current = self.current.lock().unwrap();
        *current = Some(reference);
        reference
    }

    fn clear_prompt(&mut self) {
        self.prompt = None;
        eprintln!();
        let _ = Term::stderr().hide_cursor();
    }

    fn failed(&mut self, reference: Ref) {
        {
            let bars = self.bars.lock().unwrap();
            let pb = bars.get(&reference).unwrap();
            pb.failed();
            pb.render();
        } // Drop lock

        eprintln!();

        let mut current = self.current.lock().unwrap();
        *current = None;
    }

    fn hide(&mut self, reference: Ref) {
        let bars = self.bars.lock().unwrap();
        let pb = bars.get(&reference).unwrap();
        pb.hide();
    }

    fn println(&mut self, reference: Ref, msg: &str) {
        let bars = self.bars.lock().unwrap();
        let pb = bars.get(&reference).unwrap();
        pb.println(msg);
        pb.render();
    }

    fn print_inline(&mut self, msg: &str) {
        eprintln!("{msg}");
    }

    fn prompt(&mut self, msg: &str) {
        self.prompt = Some(msg.into());
        eprint!("{}", msg);
        let _ = Term::stderr().show_cursor();
    }

    fn render(&mut self) {
        let reference = *self.current.lock().unwrap();

        if let Some(ref_id) = reference {
            let bars = self.bars.lock().unwrap();
            if let Some(pb) = bars.get(&ref_id) {
                pb.tick();
                pb.render();
            }
        }
    }

    fn set_message(&mut self, reference: Ref, msg: String) {
        {
            let bars = self.bars.lock().unwrap();
            let pb = bars.get(&reference).unwrap();
            pb.set_message(msg);
            pb.render();
        } // Drop lock
        eprintln!();
    }

    fn set_prompt_input(&mut self, input: String) {
        Term::stderr().clear_line().unwrap();
        if let Some(prompt) = &self.prompt {
            eprint!("{prompt}");
        }
        eprint!("{input}");
    }

    fn show(&mut self, reference: Ref) {
        {
            let bars = self.bars.lock().unwrap();
            let pb = bars.get(&reference).unwrap();
            pb.show();
            pb.render();
        } // Drop lock
        eprintln!();
    }

    fn succeeded(&mut self, reference: Ref) {
        {
            let bars = self.bars.lock().unwrap();
            let pb = bars.get(&reference).unwrap();
            pb.succeeded();
            pb.render();
        } // Drop lock

        eprintln!();

        let mut current = self.current.lock().unwrap();
        *current = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Progress;

    #[test]
    fn sync_new_creates_empty_progress_tracker() {
        let sync = Sync::new();
        assert_eq!(sync.bars.lock().unwrap().len(), 0);
    }

    #[test]
    fn sync_default_creates_empty_progress_tracker() {
        let sync = Sync::default();
        assert_eq!(sync.bars.lock().unwrap().len(), 0);
    }

    #[test]
    fn sync_boxed_returns_boxed_instance() {
        let _boxed_sync = Sync::boxed();
        // If this compiles and runs without panicking, the test passes
    }

    #[test]
    fn sync_append_creates_and_stores_progress_bar() {
        let mut sync = Sync::new();

        let ref1 = sync.append("Task 1");
        let ref2 = sync.append("Task 2");

        let bars = sync.bars.lock().unwrap();
        assert_eq!(bars.len(), 2);
        assert!(bars.contains_key(&ref1));
        assert!(bars.contains_key(&ref2));
        assert_ne!(ref1, ref2);
    }

    #[test]
    fn sync_append_returns_different_refs_for_different_messages() {
        let mut sync = Sync::new();

        let ref1 = sync.append("First task");
        let ref2 = sync.append("Second task");
        let ref3 = sync.append("Third task");

        assert_ne!(ref1, ref2);
        assert_ne!(ref2, ref3);
        assert_ne!(ref1, ref3);
        assert_eq!(sync.bars.lock().unwrap().len(), 3);
    }

    #[test]
    fn sync_operations_with_valid_reference() {
        let mut sync = Sync::new();
        let pb_ref = sync.append("Test task");

        // These operations should not panic with a valid reference
        sync.set_message(pb_ref, "Updated message".to_string());
        sync.println(pb_ref, "Debug output");
        sync.hide(pb_ref);
        sync.show(pb_ref);
        sync.succeeded(pb_ref);
    }

    #[test]
    fn sync_failed_operation_with_valid_reference() {
        let mut sync = Sync::new();
        let pb_ref = sync.append("Test task");

        // This operation should not panic with a valid reference
        sync.failed(pb_ref);
    }

    #[test]
    #[should_panic]
    fn sync_failed_panics_with_invalid_reference() {
        let mut sync = Sync::new();
        let invalid_ref = Ref::new(); // Create a ref that was never appended

        sync.failed(invalid_ref);
    }

    #[test]
    #[should_panic]
    fn sync_succeeded_panics_with_invalid_reference() {
        let mut sync = Sync::new();
        let invalid_ref = Ref::new();

        sync.succeeded(invalid_ref);
    }

    #[test]
    #[should_panic]
    fn sync_hide_panics_with_invalid_reference() {
        let mut sync = Sync::new();
        let invalid_ref = Ref::new();

        sync.hide(invalid_ref);
    }

    #[test]
    #[should_panic]
    fn sync_show_panics_with_invalid_reference() {
        let mut sync = Sync::new();
        let invalid_ref = Ref::new();

        sync.show(invalid_ref);
    }

    #[test]
    #[should_panic]
    fn sync_println_panics_with_invalid_reference() {
        let mut sync = Sync::new();
        let invalid_ref = Ref::new();

        sync.println(invalid_ref, "test message");
    }

    #[test]
    #[should_panic]
    fn sync_set_message_panics_with_invalid_reference() {
        let mut sync = Sync::new();
        let invalid_ref = Ref::new();

        sync.set_message(invalid_ref, "test message".to_string());
    }

    #[test]
    fn sync_multiple_append_and_operations() {
        let mut sync = Sync::new();

        let refs: Vec<_> = (0..5).map(|i| sync.append(&format!("Task {i}"))).collect();

        assert_eq!(sync.bars.lock().unwrap().len(), 5);

        // Test operations on all progress bars
        for (i, &pb_ref) in refs.iter().enumerate() {
            sync.set_message(pb_ref, format!("Updated task {i}"));
            sync.println(pb_ref, &format!("Output from task {i}"));

            if i % 2 == 0 {
                sync.succeeded(pb_ref);
            } else {
                sync.failed(pb_ref);
            }
        }
    }

    #[test]
    fn sync_implements_progress_trait() {
        fn requires_progress<T: Progress>(_: T) {}

        let sync = Sync::new();
        requires_progress(sync);
    }
}
