use crate::Progress;
use crate::progress::Ref;
use crate::render::Renderer;
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

impl std::fmt::Debug for Sync {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Sync").finish_non_exhaustive()
    }
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

    /// Looks up `reference`, panicking if it was never appended or has
    /// already finished.
    fn bar(&self, reference: Ref) -> Renderer {
        self.bars
            .lock()
            .unwrap()
            .get(&reference)
            .cloned()
            .unwrap_or_else(|| panic!("unknown progress bar {reference}"))
    }

    /// Removes a finishing bar, panicking like `bar`.
    fn take_bar(&self, reference: Ref) -> Renderer {
        self.bars
            .lock()
            .unwrap()
            .remove(&reference)
            .unwrap_or_else(|| panic!("unknown progress bar {reference}"))
    }

    /// Stops rendering `reference` if it is the current bar. A bar
    /// appended later stays current.
    fn release(&self, reference: Ref) {
        let mut current = self.current.lock().unwrap();
        if *current == Some(reference) {
            *current = None;
        }
    }

    /// The bar that owns the bottom line, unless a prompt has it.
    fn live_bar(&self) -> Option<Renderer> {
        if self.prompt.is_some() {
            return None;
        }
        let current = (*self.current.lock().unwrap())?;
        self.bars.lock().unwrap().get(&current).cloned()
    }

    /// Redraws the live bar after other output has cleared its line.
    fn redraw_live(&self) {
        if let Some(pb) = self.live_bar() {
            pb.render();
        }
    }

    /// Whether redrawing `reference` should end its line. The current bar
    /// redraws in place, and a hidden bar draws nothing, so only a visible
    /// bar that is no longer current is reprinted onto a line of its own.
    fn ends_line(&self, reference: Ref) -> bool {
        let current = *self.current.lock().unwrap();
        current != Some(reference) && self.bar(reference).is_visible()
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
        let pb = self.take_bar(reference);
        pb.failed();
        pb.render();
        if pb.is_visible() {
            eprintln!();
        }

        self.release(reference);
        self.redraw_live();
    }

    fn hide(&mut self, reference: Ref) {
        self.bar(reference).hide();
        self.redraw_live();
    }

    fn println(&mut self, reference: Ref, msg: &str) {
        self.bar(reference).println(msg);
        self.redraw_live();
    }

    fn print_inline(&mut self, msg: &str) {
        eprintln!("{msg}");
    }

    fn prompt(&mut self, msg: &str) {
        // The prompt takes over the line, so end a running bar's line first.
        if self.live_bar().is_some_and(|pb| pb.is_visible()) {
            eprintln!();
        }

        self.prompt = Some(msg.into());
        eprint!("{}", msg);
        let _ = Term::stderr().show_cursor();
    }

    fn render(&mut self) {
        if let Some(pb) = self.live_bar() {
            pb.tick();
            pb.render();
        }
    }

    fn set_message(&mut self, reference: Ref, msg: String) {
        let pb = self.bar(reference);
        pb.set_message(msg);
        pb.render();
        if self.ends_line(reference) {
            eprintln!();
            self.redraw_live();
        }
    }

    fn set_prompt_input(&mut self, input: String) {
        Term::stderr().clear_line().unwrap();
        if let Some(prompt) = &self.prompt {
            eprint!("{prompt}");
        }
        eprint!("{input}");
    }

    fn show(&mut self, reference: Ref) {
        let pb = self.bar(reference);
        pb.show();
        pb.render();
        if self.ends_line(reference) {
            eprintln!();
            self.redraw_live();
        }
    }

    fn succeeded(&mut self, reference: Ref) {
        let pb = self.take_bar(reference);
        pb.succeeded();
        pb.render();
        if pb.is_visible() {
            eprintln!();
        }

        self.release(reference);
        self.redraw_live();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Progress;

    #[test]
    fn sync_debug_trait() {
        let sync = Sync::new();
        let debug_str = format!("{sync:?}");
        assert!(debug_str.contains("Sync"));
    }

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
    fn sync_succeeded_on_stale_bar_keeps_current() {
        let mut sync = Sync::new();
        let first = sync.append("First task");
        let second = sync.append("Second task");

        sync.succeeded(first);

        assert_eq!(*sync.current.lock().unwrap(), Some(second));
    }

    #[test]
    fn sync_failed_on_stale_bar_keeps_current() {
        let mut sync = Sync::new();
        let first = sync.append("First task");
        let second = sync.append("Second task");

        sync.failed(first);

        assert_eq!(*sync.current.lock().unwrap(), Some(second));
    }

    #[test]
    fn sync_finishing_current_bar_clears_current() {
        let mut sync = Sync::new();
        let first = sync.append("First task");
        sync.succeeded(first);
        assert_eq!(*sync.current.lock().unwrap(), None);

        let second = sync.append("Second task");
        sync.failed(second);
        assert_eq!(*sync.current.lock().unwrap(), None);
    }

    #[test]
    fn sync_running_bar_redraws_in_place() {
        let mut sync = Sync::new();
        let pb = sync.append("Task");

        sync.set_message(pb, "Updated".to_string());
        sync.show(pb);

        assert!(!sync.ends_line(pb));
    }

    #[test]
    fn sync_stale_bar_ends_line_on_redraw() {
        let mut sync = Sync::new();
        let stale = sync.append("First task");
        sync.append("Second task");

        assert!(sync.ends_line(stale));
    }

    #[test]
    fn sync_hidden_bar_does_not_end_line() {
        let mut sync = Sync::new();
        let stale = sync.append("First task");
        sync.append("Second task");
        sync.hide(stale);

        assert!(!sync.ends_line(stale));
    }

    #[test]
    fn sync_finishing_removes_bar() {
        let mut sync = Sync::new();
        let first = sync.append("First task");
        let second = sync.append("Second task");

        sync.succeeded(first);
        sync.failed(second);

        assert!(sync.bars.lock().unwrap().is_empty());
    }

    #[test]
    #[should_panic(expected = "unknown progress bar")]
    fn sync_set_message_panics_after_succeeded() {
        let mut sync = Sync::new();
        let pb = sync.append("Task");
        sync.succeeded(pb);

        sync.set_message(pb, "Too late".to_string());
    }

    #[test]
    #[should_panic(expected = "unknown progress bar")]
    fn sync_println_panics_after_failed() {
        let mut sync = Sync::new();
        let pb = sync.append("Task");
        sync.failed(pb);

        sync.println(pb, "Too late");
    }

    #[test]
    #[should_panic(expected = "unknown progress bar")]
    fn sync_succeeded_panics_when_finished_twice() {
        let mut sync = Sync::new();
        let pb = sync.append("Task");
        sync.succeeded(pb);

        sync.succeeded(pb);
    }

    fn rendered(sync: &Sync, reference: Ref) -> String {
        sync.bars.lock().unwrap()[&reference].to_string()
    }

    #[test]
    fn sync_render_leaves_bar_alone_during_prompt() {
        let mut sync = Sync::new();
        let pb = sync.append("Task");
        sync.prompt("Continue? ");
        let before = rendered(&sync, pb);

        sync.render();
        sync.set_prompt_input("y".to_string());
        sync.render();

        assert_eq!(rendered(&sync, pb), before);
    }

    #[test]
    fn sync_render_resumes_after_prompt_clears() {
        let mut sync = Sync::new();
        let pb = sync.append("Task");
        sync.prompt("Continue? ");
        sync.clear_prompt();
        let before = rendered(&sync, pb);

        sync.render();

        assert_ne!(rendered(&sync, pb), before);
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
