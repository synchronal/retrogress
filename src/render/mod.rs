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
