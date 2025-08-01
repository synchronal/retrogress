use crate::progress::Ref;
use crate::render::{MultiProgress, ProgressBar};
use crate::Progress;

use console::style;
use std::collections::HashMap;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};

#[derive(Debug)]
enum ProgressMessage {
    Append { reference: Ref, message: String },
    Println { reference: Ref, message: String },
    SetMessage { reference: Ref, message: String },
    Failed { reference: Ref },
    Succeeded { reference: Ref },
    Hide { reference: Ref },
    Show { reference: Ref },
    Shutdown,
}

struct ProgressBarState {
    bar: ProgressBar,
    output_buffer: Vec<String>,
}

/// An implementation of `Progress` designed for parallel execution
/// where multiple progress bars may output simultaneously.
///
/// Uses indicatif's MultiProgress to coordinate all progress bars and output
/// through a single background thread, preventing output collision.
///
/// ```rust
/// use retrogress::{ProgressBar, Parallel};
/// let mut progress = ProgressBar::new(Parallel::boxed());
/// ```
pub struct Parallel {
    message_sender: Sender<ProgressMessage>,
    worker_thread: Option<JoinHandle<()>>,
}

impl Parallel {
    pub fn new() -> Self {
        console::set_colors_enabled(true);
        console::set_colors_enabled_stderr(true);

        let (sender, receiver) = mpsc::channel();
        let worker_thread = Some(thread::spawn(move || {
            Self::progress_worker(receiver);
        }));

        Self {
            message_sender: sender,
            worker_thread,
        }
    }

    pub fn boxed() -> Box<Self> {
        Box::new(Self::new())
    }

    fn progress_worker(receiver: Receiver<ProgressMessage>) {
        let multi_progress = MultiProgress::new();
        let mut bars: HashMap<Ref, ProgressBarState> = HashMap::new();

        while let Ok(message) = receiver.recv() {
            match message {
                ProgressMessage::Append { reference, message } => {
                    let pb = ProgressBar::new(message);
                    let bar = multi_progress.add(pb);

                    bars.insert(
                        reference,
                        ProgressBarState {
                            bar,
                            output_buffer: Vec::new(),
                        },
                    );
                }
                ProgressMessage::Println { reference, message } => {
                    if let Some(state) = bars.get_mut(&reference) {
                        // Store in buffer and print immediately
                        state.output_buffer.push(message.clone());
                        state.bar.println(&message);
                    }
                }
                ProgressMessage::SetMessage { reference, message } => {
                    if let Some(state) = bars.get(&reference) {
                        state.bar.set_message(message);
                    }
                }
                ProgressMessage::Failed { reference } => {
                    if let Some(state) = bars.get(&reference) {
                        state
                            .bar
                            .set_prefix(format!("{}", style("𝗑").bold().bright().red()));
                        state.bar.finish();
                    }
                }
                ProgressMessage::Succeeded { reference } => {
                    if let Some(state) = bars.get(&reference) {
                        state
                            .bar
                            .set_prefix(format!("{}", style("✓").bold().green()));
                        state.bar.finish();
                    }
                }
                ProgressMessage::Hide { reference } => {
                    if let Some(state) = bars.get(&reference) {
                        state.bar.hide();
                    }
                }
                ProgressMessage::Show { reference } => {
                    if let Some(state) = bars.get(&reference) {
                        state.bar.show();
                    }
                }
                ProgressMessage::Shutdown => {
                    break;
                }
            }
        }

        // Clear any remaining progress bars
        multi_progress.clear().ok();
    }
}

impl Default for Parallel {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Parallel {
    fn drop(&mut self) {
        // Signal shutdown to worker thread
        let _ = self.message_sender.send(ProgressMessage::Shutdown);

        // Wait for worker thread to finish
        if let Some(handle) = self.worker_thread.take() {
            let _ = handle.join();
        }
    }
}

impl Progress for Parallel {
    fn append(&mut self, msg: &str) -> Ref {
        let reference = Ref::new();
        let _ = self.message_sender.send(ProgressMessage::Append {
            reference,
            message: msg.to_string(),
        });
        reference
    }

    fn failed(&mut self, reference: Ref) {
        let _ = self
            .message_sender
            .send(ProgressMessage::Failed { reference });
    }

    fn hide(&mut self, reference: Ref) {
        let _ = self
            .message_sender
            .send(ProgressMessage::Hide { reference });
    }

    fn println(&mut self, reference: Ref, msg: &str) {
        let _ = self.message_sender.send(ProgressMessage::Println {
            reference,
            message: msg.to_string(),
        });
    }

    fn set_message(&mut self, reference: Ref, msg: String) {
        let _ = self.message_sender.send(ProgressMessage::SetMessage {
            reference,
            message: msg,
        });
    }

    fn show(&mut self, reference: Ref) {
        let _ = self
            .message_sender
            .send(ProgressMessage::Show { reference });
    }

    fn succeeded(&mut self, reference: Ref) {
        let _ = self
            .message_sender
            .send(ProgressMessage::Succeeded { reference });
    }
}
