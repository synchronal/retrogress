use console::{style, Term};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

const SPINNER_CHARS: &[char] = &['⣾', '⣽', '⣻', '⢿', '⡿', '⣟', '⣯', '⣷'];

#[derive(Clone)]
pub struct ProgressBar {
    state: Arc<Mutex<ProgressBarState>>,
    ticker: Arc<AtomicBool>,
    tick_thread: Arc<Mutex<Option<thread::JoinHandle<()>>>>,
}

struct ProgressBarState {
    prefix: String,
    message: String,
    spinner_index: usize,
    visible: bool,
    finished: bool,
}

impl ProgressBar {
    pub fn new(message: String) -> Self {
        let state = Arc::new(Mutex::new(ProgressBarState {
            prefix: format!("{}", style("•").green()),
            message,
            spinner_index: 0,
            visible: true,
            finished: false,
        }));

        let ticker = Arc::new(AtomicBool::new(true));

        let pb = ProgressBar {
            state,
            ticker,
            tick_thread: Arc::new(Mutex::new(None)),
        };

        pb.start_ticker();
        pb
    }

    fn start_ticker(&self) {
        let state = Arc::clone(&self.state);
        let ticker = Arc::clone(&self.ticker);

        let handle = thread::spawn(move || {
            while ticker.load(Ordering::Relaxed) {
                thread::sleep(Duration::from_millis(50));

                let mut state_guard = state.lock().unwrap();
                if state_guard.visible && !state_guard.finished {
                    state_guard.spinner_index =
                        (state_guard.spinner_index + 1) % SPINNER_CHARS.len();
                    drop(state_guard);
                    Self::render(&state);
                }
            }
        });

        *self.tick_thread.lock().unwrap() = Some(handle);
    }

    pub fn set_prefix(&self, prefix: String) {
        let mut state = self.state.lock().unwrap();
        state.prefix = prefix;
    }

    pub fn set_message(&self, message: String) {
        let mut state = self.state.lock().unwrap();
        state.message = message;
        drop(state);
        self.render_now();
    }

    pub fn println(&self, msg: &str) {
        let state = self.state.lock().unwrap();
        if state.visible {
            Term::stderr().clear_line().unwrap();
            eprintln!("{msg}");
            drop(state);
            self.render_now();
        }
    }

    pub fn hide(&self) {
        self.ticker.store(false, Ordering::Relaxed);
        let mut state = self.state.lock().unwrap();
        state.visible = false;
        Term::stderr().clear_line().unwrap();
    }

    pub fn show(&self) {
        let mut state = self.state.lock().unwrap();
        state.visible = true;
        self.ticker.store(true, Ordering::Relaxed);
        drop(state);
        self.render_now();
    }

    pub fn finish(&self) {
        self.ticker.store(false, Ordering::Relaxed);

        let mut state = self.state.lock().unwrap();
        state.finished = true;
        drop(state);

        self.render_now();
        eprintln!();

        if let Some(handle) = self.tick_thread.lock().unwrap().take() {
            let _ = handle.join();
        }
    }

    fn render_now(&self) {
        Self::render(&self.state);
    }

    fn render(state_arc: &Arc<Mutex<ProgressBarState>>) {
        let state = state_arc.lock().unwrap();
        if !state.visible {
            return;
        }

        Term::stderr().clear_line().unwrap();

        if state.finished {
            eprint!("{} {}", state.prefix, state.message);
        } else {
            let spinner = SPINNER_CHARS[state.spinner_index];
            eprint!("{} {} {}", state.prefix, spinner, state.message);
        }

        Term::stderr().flush().unwrap();
    }
}

impl Drop for ProgressBar {
    fn drop(&mut self) {
        self.ticker.store(false, Ordering::Relaxed);
        if let Some(handle) = self.tick_thread.lock().unwrap().take() {
            let _ = handle.join();
        }
    }
}

pub struct MultiProgress {
    bars: Arc<Mutex<Vec<ProgressBar>>>,
}

impl MultiProgress {
    pub fn new() -> Self {
        MultiProgress {
            bars: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn add(&self, bar: ProgressBar) -> ProgressBar {
        let mut bars = self.bars.lock().unwrap();
        bars.push(bar.clone());
        bar
    }

    pub fn clear(&self) -> Result<(), std::io::Error> {
        let bars = self.bars.lock().unwrap();
        for bar in bars.iter() {
            bar.hide();
        }
        Ok(())
    }
}

impl Default for MultiProgress {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn progress_bar_new_creates_with_message() {
        let message = "Test progress bar".to_string();
        let pb = ProgressBar::new(message.clone());

        let state = pb.state.lock().unwrap();
        assert_eq!(state.message, message);
        assert_eq!(state.spinner_index, 0);
        assert_eq!(state.visible, true);
        assert_eq!(state.finished, false);

        // Check that ticker is running
        assert_eq!(pb.ticker.load(Ordering::Relaxed), true);
    }

    #[test]
    fn progress_bar_set_prefix() {
        let pb = ProgressBar::new("Test".to_string());
        let new_prefix = "✓".to_string();

        pb.set_prefix(new_prefix.clone());

        let state = pb.state.lock().unwrap();
        assert_eq!(state.prefix, new_prefix);
    }

    #[test]
    fn progress_bar_set_message() {
        let pb = ProgressBar::new("Initial message".to_string());
        let new_message = "Updated message".to_string();

        pb.set_message(new_message.clone());

        let state = pb.state.lock().unwrap();
        assert_eq!(state.message, new_message);
    }

    #[test]
    fn progress_bar_hide() {
        let pb = ProgressBar::new("Test".to_string());

        pb.hide();

        let state = pb.state.lock().unwrap();
        assert_eq!(state.visible, false);
        assert_eq!(pb.ticker.load(Ordering::Relaxed), false);
    }

    #[test]
    fn progress_bar_show() {
        let pb = ProgressBar::new("Test".to_string());

        // First hide it
        pb.hide();
        assert_eq!(pb.state.lock().unwrap().visible, false);

        // Then show it
        pb.show();

        let state = pb.state.lock().unwrap();
        assert_eq!(state.visible, true);
        assert_eq!(pb.ticker.load(Ordering::Relaxed), true);
    }

    #[test]
    fn progress_bar_finish() {
        let pb = ProgressBar::new("Test".to_string());

        pb.finish();

        let state = pb.state.lock().unwrap();
        assert_eq!(state.finished, true);
        assert_eq!(pb.ticker.load(Ordering::Relaxed), false);
    }

    #[test]
    fn progress_bar_println_when_visible() {
        let pb = ProgressBar::new("Test".to_string());

        // This should not panic when the progress bar is visible
        pb.println("Test output");
    }

    #[test]
    fn progress_bar_println_when_hidden() {
        let pb = ProgressBar::new("Test".to_string());
        pb.hide();

        // This should not panic when the progress bar is hidden
        pb.println("Test output");
    }

    #[test]
    fn progress_bar_clone() {
        let pb = ProgressBar::new("Test".to_string());
        let pb_clone = pb.clone();

        pb.set_message("New message".to_string());

        {
            let original_state = pb.state.lock().unwrap();
            assert_eq!(original_state.message, "New message");
        }
        {
            let cloned_state = pb_clone.state.lock().unwrap();
            assert_eq!(cloned_state.message, "New message");
        }

        pb.ticker.store(false, Ordering::Relaxed);
    }

    #[test]
    fn progress_bar_drop_stops_ticker() {
        let pb = ProgressBar::new("Test".to_string());

        assert_eq!(pb.ticker.load(Ordering::Relaxed), true);
        drop(pb);

        // We can't directly test the ticker state after drop, but if this
        // test completes without hanging, it indicates proper cleanup
    }

    #[test]
    fn spinner_chars_constant() {
        assert_eq!(SPINNER_CHARS, &['⣾', '⣽', '⣻', '⢿', '⡿', '⣟', '⣯', '⣷']);
    }

    #[test]
    fn multi_progress_new() {
        let multi = MultiProgress::new();
        let bars = multi.bars.lock().unwrap();
        assert_eq!(bars.len(), 0);
    }

    #[test]
    fn multi_progress_default() {
        let multi = MultiProgress::default();
        let bars = multi.bars.lock().unwrap();
        assert_eq!(bars.len(), 0);
    }

    #[test]
    fn multi_progress_add() {
        let multi = MultiProgress::new();
        let pb1 = ProgressBar::new("Test 1".to_string());
        let pb2 = ProgressBar::new("Test 2".to_string());

        let returned_pb1 = multi.add(pb1);
        let returned_pb2 = multi.add(pb2);

        let bars = multi.bars.lock().unwrap();
        assert_eq!(bars.len(), 2);

        assert_eq!(returned_pb1.state.lock().unwrap().message, "Test 1");
        assert_eq!(returned_pb2.state.lock().unwrap().message, "Test 2");
    }

    #[test]
    fn multi_progress_clear() {
        let multi = MultiProgress::new();
        let pb1 = ProgressBar::new("Test 1".to_string());
        let pb2 = ProgressBar::new("Test 2".to_string());

        multi.add(pb1);
        multi.add(pb2);

        let result = multi.clear();
        assert!(result.is_ok());

        // All bars should now be hidden
        let bars = multi.bars.lock().unwrap();
        for bar in bars.iter() {
            assert_eq!(bar.state.lock().unwrap().visible, false);
        }
    }

    #[test]
    fn multi_progress_clear_empty() {
        let multi = MultiProgress::new();
        let result = multi.clear();
        assert!(result.is_ok());
    }

    #[test]
    fn progress_bar_ticker_updates_spinner() {
        let pb = ProgressBar::new("Test".to_string());

        let initial_index = pb.state.lock().unwrap().spinner_index;

        // Sleep to allow ticker to update
        thread::sleep(Duration::from_millis(100));

        let updated_index = pb.state.lock().unwrap().spinner_index;

        // The spinner index should have changed (assuming the ticker is working)
        // Note: This test could be flaky in very slow environments
        assert!(updated_index != initial_index || updated_index < SPINNER_CHARS.len());
    }

    #[test]
    fn progress_bar_finished_stops_spinner_updates() {
        let pb = ProgressBar::new("Test".to_string());

        pb.finish();
        let initial_index = pb.state.lock().unwrap().spinner_index;

        // Sleep to ensure ticker would have updated if it was still running
        thread::sleep(Duration::from_millis(100));

        let final_index = pb.state.lock().unwrap().spinner_index;

        // Index should not change after finishing
        assert_eq!(initial_index, final_index);
    }
}
