use console::{style, Term};
use std::sync::{Arc, Mutex};

const SPINNER_CHARS: &[char] = &['⣾', '⣽', '⣻', '⢿', '⡿', '⣟', '⣯', '⣷'];

#[derive(Clone)]
pub struct Renderer {
    state: Arc<Mutex<RendererState>>,
}

struct RendererState {
    prefix: String,
    message: String,
    spinner_index: usize,
    visible: bool,
    finished: bool,
}

impl Renderer {
    pub fn new(message: String) -> Self {
        let state = Arc::new(Mutex::new(RendererState {
            prefix: format!("{}", style("•").green()),
            message,
            spinner_index: 0,
            visible: true,
            finished: false,
        }));

        Renderer { state }
    }

    pub fn set_prefix(&self, prefix: String) {
        let mut state = self.state.lock().unwrap();
        state.prefix = prefix;
    }

    pub fn set_message(&self, message: String) {
        let mut state = self.state.lock().unwrap();
        state.message = message;
        drop(state);
        self.render();
    }

    pub fn println(&self, msg: &str) {
        let state = self.state.lock().unwrap();
        if state.visible {
            Term::stderr().clear_line().unwrap();
            eprintln!("{msg}");
            drop(state);
            self.render();
        }
    }

    pub fn hide(&self) {
        let mut state = self.state.lock().unwrap();
        state.visible = false;
        Term::stderr().clear_line().unwrap();
    }

    pub fn show(&self) {
        let mut state = self.state.lock().unwrap();
        state.visible = true;
        drop(state);
        self.render();
    }

    pub fn finish(&self) {
        let mut state = self.state.lock().unwrap();
        state.finished = true;
        drop(state);

        self.render();
        eprintln!();
    }

    pub fn tick(&self) {
        let mut state = self.state.lock().unwrap();
        if state.visible && !state.finished {
            state.spinner_index = (state.spinner_index + 1) % SPINNER_CHARS.len();
        }
    }

    pub fn render(&self) {
        let state = self.state.lock().unwrap();
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

pub struct MultiProgress {
    bars: Arc<Mutex<Vec<Renderer>>>,
}

impl MultiProgress {
    pub fn new() -> Self {
        MultiProgress {
            bars: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn add(&self, bar: Renderer) -> Renderer {
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

    #[test]
    fn progress_bar_new_creates_with_message() {
        let message = "Test progress bar".to_string();
        let pb = Renderer::new(message.clone());

        let state = pb.state.lock().unwrap();
        assert_eq!(state.message, message);
        assert_eq!(state.spinner_index, 0);
        assert_eq!(state.visible, true);
        assert_eq!(state.finished, false);
    }

    #[test]
    fn progress_bar_set_prefix() {
        let pb = Renderer::new("Test".to_string());
        let new_prefix = "✓".to_string();

        pb.set_prefix(new_prefix.clone());

        let state = pb.state.lock().unwrap();
        assert_eq!(state.prefix, new_prefix);
    }

    #[test]
    fn progress_bar_set_message() {
        let pb = Renderer::new("Initial message".to_string());
        let new_message = "Updated message".to_string();

        pb.set_message(new_message.clone());

        let state = pb.state.lock().unwrap();
        assert_eq!(state.message, new_message);
    }

    #[test]
    fn progress_bar_hide() {
        let pb = Renderer::new("Test".to_string());

        pb.hide();

        let state = pb.state.lock().unwrap();
        assert_eq!(state.visible, false);
    }

    #[test]
    fn progress_bar_show() {
        let pb = Renderer::new("Test".to_string());

        // First hide it
        pb.hide();
        assert_eq!(pb.state.lock().unwrap().visible, false);

        // Then show it
        pb.show();

        let state = pb.state.lock().unwrap();
        assert_eq!(state.visible, true);
    }

    #[test]
    fn progress_bar_finish() {
        let pb = Renderer::new("Test".to_string());

        pb.finish();

        let state = pb.state.lock().unwrap();
        assert_eq!(state.finished, true);
    }

    #[test]
    fn progress_bar_println_when_visible() {
        let pb = Renderer::new("Test".to_string());

        // This should not panic when the progress bar is visible
        pb.println("Test output");
    }

    #[test]
    fn progress_bar_println_when_hidden() {
        let pb = Renderer::new("Test".to_string());
        pb.hide();

        // This should not panic when the progress bar is hidden
        pb.println("Test output");
    }

    #[test]
    fn progress_bar_clone() {
        let pb = Renderer::new("Test".to_string());
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
        let pb1 = Renderer::new("Test 1".to_string());
        let pb2 = Renderer::new("Test 2".to_string());

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
        let pb1 = Renderer::new("Test 1".to_string());
        let pb2 = Renderer::new("Test 2".to_string());

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
    fn progress_bar_tick_updates_spinner() {
        let pb = Renderer::new("Test".to_string());

        let initial_index = pb.state.lock().unwrap().spinner_index;

        // Call tick to update spinner
        pb.tick();

        let updated_index = pb.state.lock().unwrap().spinner_index;

        // The spinner index should have changed
        assert_eq!(updated_index, (initial_index + 1) % SPINNER_CHARS.len());
    }

    #[test]
    fn progress_bar_finished_stops_spinner_updates() {
        let pb = Renderer::new("Test".to_string());

        pb.finish();
        let initial_index = pb.state.lock().unwrap().spinner_index;

        // Call tick - it should not update the spinner after finish
        pb.tick();

        let final_index = pb.state.lock().unwrap().spinner_index;

        // Index should not change after finishing
        assert_eq!(initial_index, final_index);
    }
}
