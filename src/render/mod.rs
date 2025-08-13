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

impl std::fmt::Display for RendererState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.visible {
            return write!(f, "");
        }

        if self.finished {
            write!(f, "{} {}", self.prefix, self.message)
        } else {
            let spinner = SPINNER_CHARS[self.spinner_index];
            write!(f, "{} {} {}", self.prefix, spinner, self.message)
        }
    }
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

    pub fn failed(&self) {
        self.set_prefix(format!("{}", style("𝗑").bold().bright().red()));
        self.finish();
    }

    pub fn set_prefix(&self, prefix: String) {
        let mut state = self.state.lock().unwrap();
        state.prefix = prefix;
    }

    pub fn set_message(&self, message: String) {
        let mut state = self.state.lock().unwrap();
        state.message = message;
        drop(state);
    }

    pub fn println(&self, msg: &str) {
        let state = self.state.lock().unwrap();
        if state.visible {
            Term::stderr().clear_line().unwrap();
            eprintln!("{msg}");
            drop(state);
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
    }

    pub fn finish(&self) {
        let mut state = self.state.lock().unwrap();
        state.finished = true;
        drop(state);
    }

    pub fn succeeded(&self) {
        self.set_prefix(format!("{}", style("✓").bold().green()));
        self.finish();
    }

    pub fn tick(&self) {
        let mut state = self.state.lock().unwrap();
        if state.visible && !state.finished {
            state.spinner_index = (state.spinner_index + 1) % SPINNER_CHARS.len();
        }
    }

    pub fn debug_state(&self) -> (bool, String) {
        let state = self.state.lock().unwrap();
        (state.finished, state.prefix.clone())
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

impl std::fmt::Display for Renderer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let state = self.state.lock().unwrap();
        write!(f, "{state}")
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
        assert!(state.visible);
        assert!(!state.finished);
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
        assert!(!state.visible);
    }

    #[test]
    fn progress_bar_show() {
        let pb = Renderer::new("Test".to_string());

        // First hide it
        pb.hide();
        assert!(!pb.state.lock().unwrap().visible);

        // Then show it
        pb.show();

        let state = pb.state.lock().unwrap();
        assert!(state.visible);
    }

    #[test]
    fn progress_bar_finish() {
        let pb = Renderer::new("Test".to_string());

        pb.finish();

        let state = pb.state.lock().unwrap();
        assert!(state.finished);
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
