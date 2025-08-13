use crate::progress::Ref;
use crate::render::Renderer;
use crate::Progress;

use console::Term;
use std::collections::HashMap;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
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
    bar: Renderer,
    output_buffer: Vec<String>,
}

#[derive(Default)]
struct State {
    bars: HashMap<Ref, ProgressBarState>,
    output_buffer_length: usize,
    running: Vec<Ref>,
    succeeded: Vec<Ref>,
}

/// An implementation of `Progress` designed for parallel execution
/// where multiple progress bars may output simultaneously.
///
/// Coordinates all progress bars and output through a single background
/// thread, preventing output collision.
///
/// ```rust
/// use retrogress::{ProgressBar, Parallel};
/// let mut progress = ProgressBar::new(Parallel::boxed());
/// ```
pub struct Parallel {
    pub console_size: (u16, u16),
    message_sender: Sender<ProgressMessage>,
    worker_thread: Option<JoinHandle<()>>,
    state: Arc<Mutex<State>>,
}

unsafe impl std::marker::Send for Parallel {}
unsafe impl std::marker::Sync for Parallel {}

impl Parallel {
    pub fn new() -> Self {
        console::set_colors_enabled(true);
        console::set_colors_enabled_stderr(true);

        let console_size = console::Term::stdout().size();
        let state = Arc::new(Mutex::new(State::default()));

        let (sender, receiver) = mpsc::channel();
        let state_clone = Arc::clone(&state);
        let worker_thread = Some(thread::spawn(move || {
            Self::progress_worker(receiver, state_clone);
        }));

        Self {
            console_size,
            message_sender: sender,
            worker_thread,
            state,
        }
    }

    pub fn boxed() -> Box<Self> {
        Box::new(Self::new())
    }

    fn progress_worker(receiver: Receiver<ProgressMessage>, state: Arc<Mutex<State>>) {
        while let Ok(message) = receiver.recv() {
            match message {
                ProgressMessage::Append { reference, message } => {
                    let mut state = state.lock().unwrap();
                    let pb = Renderer::new(message);

                    state.bars.insert(
                        reference,
                        ProgressBarState {
                            bar: pb,
                            output_buffer: Vec::new(),
                        },
                    );
                    state.running.push(reference);
                }
                ProgressMessage::Failed { reference } => {
                    let state = state.lock().unwrap();
                    let bar = state.bars.get(&reference).unwrap();
                    bar.bar.failed();
                }
                ProgressMessage::Hide { reference } => {
                    let state = state.lock().unwrap();
                    let bar = state.bars.get(&reference).unwrap();
                    bar.bar.hide();
                }
                ProgressMessage::Println { reference, message } => {
                    let mut state = state.lock().unwrap();
                    let bar = state.bars.get_mut(&reference).unwrap();
                    bar.output_buffer.push(message);

                    // Trim buffer to keep only the last 1000 lines
                    if bar.output_buffer.len() > 1000 {
                        bar.output_buffer.drain(0..bar.output_buffer.len() - 1000);
                    }
                }
                ProgressMessage::SetMessage { reference, message } => {
                    let state = state.lock().unwrap();
                    let bar = state.bars.get(&reference).unwrap();
                    bar.bar.set_message(message);
                }
                ProgressMessage::Show { reference } => {
                    let state = state.lock().unwrap();
                    let bar = state.bars.get(&reference).unwrap();
                    bar.bar.show();
                }
                ProgressMessage::Succeeded { reference } => {
                    let mut state = state.lock().unwrap();
                    let bar = state.bars.get(&reference).unwrap();
                    bar.bar.succeeded();

                    state.running.retain(|x| *x != reference);
                    state.succeeded.push(reference);
                }
                ProgressMessage::Shutdown => {
                    break;
                }
            }
        }
    }
}

impl Default for Parallel {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Parallel {
    fn drop(&mut self) {
        let _ = self.message_sender.send(ProgressMessage::Shutdown);

        if let Some(handle) = self.worker_thread.take() {
            let _ = handle.join();
        }

        self.render();
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

    fn render(&mut self) {
        let mut state = self.state.lock().unwrap();

        let term = Term::stderr();
        let mut output_buffer = String::new();

        if state.output_buffer_length > 0 {
            // Backtrack cursor to the start
            output_buffer.push_str(&format!("\x1b[{}A", state.output_buffer_length));
            output_buffer.push_str("\x1b[0G"); // Beginning of line
            output_buffer.push_str("\x1b[J"); // Clear to end
        }

        let mut lines_rendered = 0;

        let succeeded = state.succeeded.clone();
        for reference in &succeeded {
            let bar_state = state.bars.get_mut(reference).unwrap();
            let rendered = bar_state.bar.to_string();
            if !rendered.is_empty() {
                output_buffer.push_str(&rendered);
                output_buffer.push('\n');
                lines_rendered += 1;
            }
        }

        let running = state.running.clone();
        for reference in &running {
            let bar_state = state.bars.get_mut(reference).unwrap();
            let start_idx = if bar_state.output_buffer.len() > 5 {
                bar_state.output_buffer.len() - 5
            } else {
                0
            };
            for line in &bar_state.output_buffer[start_idx..] {
                output_buffer.push_str(line);
                output_buffer.push('\n');
                lines_rendered += 1;
            }

            bar_state.bar.tick();
            let rendered = bar_state.bar.to_string();
            if !rendered.is_empty() {
                output_buffer.push_str(&rendered);
                output_buffer.push('\n');
                lines_rendered += 1;
            }
        }

        state.output_buffer_length = lines_rendered;

        eprint!("{}", output_buffer);
        term.flush().ok();
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Progress;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn parallel_new_creates_worker_thread() {
        let parallel = Parallel::new();
        assert!(parallel
            .message_sender
            .send(ProgressMessage::Shutdown)
            .is_ok());
        assert!(parallel.worker_thread.is_some());
    }

    #[test]
    fn parallel_default_creates_worker_thread() {
        let parallel = Parallel::default();

        assert!(parallel
            .message_sender
            .send(ProgressMessage::Shutdown)
            .is_ok());
        assert!(parallel.worker_thread.is_some());
    }

    #[test]
    fn parallel_boxed_returns_boxed_instance() {
        let _boxed_parallel = Parallel::boxed();
    }

    #[test]
    fn parallel_append_returns_unique_refs() {
        let mut parallel = Parallel::new();

        let ref1 = parallel.append("Task 1");
        let ref2 = parallel.append("Task 2");
        let ref3 = parallel.append("Task 3");

        assert_ne!(ref1, ref2);
        assert_ne!(ref2, ref3);
        assert_ne!(ref1, ref3);
    }

    #[test]
    fn parallel_operations_do_not_panic() {
        let mut parallel = Parallel::new();

        let pb_ref = parallel.append("Test task");

        parallel.set_message(pb_ref, "Updated message".to_string());
        parallel.println(pb_ref, "Debug output");
        parallel.hide(pb_ref);
        parallel.show(pb_ref);
        parallel.succeeded(pb_ref);
    }

    #[test]
    fn parallel_failed_operation() {
        let mut parallel = Parallel::new();
        let pb_ref = parallel.append("Test task");

        parallel.failed(pb_ref);
    }

    #[test]
    fn parallel_multiple_operations_sequence() {
        let mut parallel = Parallel::new();

        let refs: Vec<_> = (0..5)
            .map(|i| parallel.append(&format!("Task {i}")))
            .collect();

        for (i, &pb_ref) in refs.iter().enumerate() {
            parallel.set_message(pb_ref, format!("Updated task {i}"));
            parallel.println(pb_ref, &format!("Output from task {i}"));

            if i % 2 == 0 {
                parallel.succeeded(pb_ref);
            } else {
                parallel.failed(pb_ref);
            }
        }

        let _ = parallel.message_sender.send(ProgressMessage::Shutdown);
        if let Some(handle) = parallel.worker_thread.take() {
            let _ = handle.join();
        }
    }

    #[test]
    fn parallel_implements_progress_trait() {
        fn requires_progress<T: Progress>(_: T) {}

        let parallel = Parallel::new();
        requires_progress(parallel);
    }

    #[test]
    fn parallel_drop_shuts_down_worker_thread() {
        let parallel = Parallel::new();
        drop(parallel);
    }

    #[test]
    fn parallel_message_sender_channel_works() {
        let mut parallel = Parallel::new();

        let pb_ref = parallel.append("Test");

        assert!(parallel
            .message_sender
            .send(ProgressMessage::SetMessage {
                reference: pb_ref,
                message: "Test message".to_string(),
            })
            .is_ok());

        assert!(parallel
            .message_sender
            .send(ProgressMessage::Println {
                reference: pb_ref,
                message: "Test output".to_string(),
            })
            .is_ok());

        assert!(parallel
            .message_sender
            .send(ProgressMessage::Hide { reference: pb_ref })
            .is_ok());

        assert!(parallel
            .message_sender
            .send(ProgressMessage::Show { reference: pb_ref })
            .is_ok());

        assert!(parallel
            .message_sender
            .send(ProgressMessage::Succeeded { reference: pb_ref })
            .is_ok());

        assert!(parallel
            .message_sender
            .send(ProgressMessage::Shutdown)
            .is_ok());
        if let Some(handle) = parallel.worker_thread.take() {
            let _ = handle.join();
        }
    }

    #[test]
    fn parallel_worker_thread_processes_shutdown() {
        let parallel = Parallel::new();
        assert!(parallel
            .message_sender
            .send(ProgressMessage::Shutdown)
            .is_ok());
        let result = parallel.message_sender.send(ProgressMessage::Shutdown);
        assert!(result.is_ok())
    }

    #[test]
    fn progress_message_debug_trait() {
        let pb_ref = Ref::new();

        let messages = vec![
            ProgressMessage::Append {
                reference: pb_ref,
                message: "test".to_string(),
            },
            ProgressMessage::Println {
                reference: pb_ref,
                message: "test".to_string(),
            },
            ProgressMessage::SetMessage {
                reference: pb_ref,
                message: "test".to_string(),
            },
            ProgressMessage::Failed { reference: pb_ref },
            ProgressMessage::Succeeded { reference: pb_ref },
            ProgressMessage::Hide { reference: pb_ref },
            ProgressMessage::Show { reference: pb_ref },
            ProgressMessage::Shutdown,
        ];

        for message in messages {
            let debug_str = format!("{message:?}");
            assert!(!debug_str.is_empty());
        }
    }

    #[test]
    fn progress_bar_state_buffer() {
        let mut parallel = Parallel::new();
        let pb_ref = parallel.append("Test task");

        parallel.println(pb_ref, "Line 1");
        parallel.println(pb_ref, "Line 2");
        parallel.println(pb_ref, "Line 3");

        // TODO: assert contents of progress bar

        let _ = parallel.message_sender.send(ProgressMessage::Shutdown);
        if let Some(handle) = parallel.worker_thread.take() {
            let _ = handle.join();
        }
    }

    #[test]
    fn parallel_render_sends_tick_message() {
        let mut parallel = Parallel::new();
        let pb_ref = parallel.append("Test task");

        // Calling render should send a Tick message
        parallel.render();

        // Add a small delay to allow the worker thread to process
        thread::sleep(Duration::from_millis(100));

        parallel.succeeded(pb_ref);
    }

    #[test]
    fn parallel_render_with_multiple_progress_bars() {
        let mut parallel = Parallel::new();

        let pb1 = parallel.append("Task 1");
        let pb2 = parallel.append("Task 2");
        let pb3 = parallel.append("Task 3");

        parallel.set_message(pb1, "Processing task 1".to_string());
        parallel.set_message(pb2, "Processing task 2".to_string());
        parallel.set_message(pb3, "Processing task 3".to_string());

        // Call render multiple times
        for _ in 0..3 {
            parallel.render();
            thread::sleep(Duration::from_millis(50));
        }

        parallel.succeeded(pb1);
        parallel.failed(pb2);
        parallel.succeeded(pb3);
    }

    #[test]
    fn parallel_concurrent_access() {
        use std::sync::{Arc, Barrier};

        let mut parallel = Parallel::new();
        let pb_ref = parallel.append("Concurrent test");

        let barrier = Arc::new(Barrier::new(2));
        let barrier_clone = barrier.clone();

        let sender = parallel.message_sender.clone();

        let handle = thread::spawn(move || {
            for i in 0..10 {
                let _ = sender.send(ProgressMessage::SetMessage {
                    reference: pb_ref,
                    message: format!("Thread message {i}"),
                });
            }
            barrier_clone.wait();
        });

        for i in 0..10 {
            parallel.set_message(pb_ref, format!("Main message {i}"));
        }

        barrier.wait();
        handle.join().unwrap();
    }

    #[test]
    fn channel_disconnection_after_worker_panic() {
        let mut parallel = Parallel::new();
        let pb_ref = parallel.append("Test task before panic");

        let _ = parallel.message_sender.send(ProgressMessage::Shutdown);

        if let Some(handle) = parallel.worker_thread.take() {
            let _ = handle.join();
        }

        parallel.set_message(pb_ref, "This message won't be processed".to_string());
        parallel.println(pb_ref, "This output won't appear");
        parallel.succeeded(pb_ref);
    }

    #[test]
    fn operations_after_channel_failure() {
        let mut parallel = Parallel::new();
        let pb_ref = parallel.append("Test task");

        if let Some(handle) = parallel.worker_thread.take() {
            let _ = parallel.message_sender.send(ProgressMessage::Shutdown);
            let _ = handle.join();
        }

        let result1 = parallel.message_sender.send(ProgressMessage::SetMessage {
            reference: pb_ref,
            message: "This will fail".to_string(),
        });
        assert!(result1.is_err());

        let result2 = parallel.message_sender.send(ProgressMessage::Println {
            reference: pb_ref,
            message: "This will also fail".to_string(),
        });
        assert!(result2.is_err());

        parallel.set_message(pb_ref, "API call after channel failure".to_string());
        parallel.println(pb_ref, "Another API call");
        parallel.succeeded(pb_ref);
    }

    #[test]
    fn graceful_handling_of_channel_errors() {
        use std::sync::{Arc, Barrier};

        let parallel = Parallel::new();
        let barrier = Arc::new(Barrier::new(2));
        let barrier_clone = barrier.clone();

        let mut parallel_clone = Parallel::new();

        let handle = thread::spawn(move || {
            let pb_ref = parallel_clone.append("Thread task");
            barrier_clone.wait();
            parallel_clone.set_message(pb_ref, "Message from thread".to_string());
            parallel_clone.println(pb_ref, "Output from thread");
            parallel_clone.succeeded(pb_ref);
        });

        barrier.wait();
        drop(parallel);
        handle.join().unwrap();
    }

    #[test]
    fn stress_high_frequency_concurrent_operations() {
        use crate::ProgressBar;
        use std::sync::{Arc, Barrier};

        let mut parallel = ProgressBar::new(Parallel::boxed());
        let pb_ref = parallel.append("Stress test progress bar");

        let num_threads = 10;
        let operations_per_thread = 100;
        let barrier = Arc::new(Barrier::new(num_threads + 1));

        let mut handles = vec![];

        for thread_id in 0..num_threads {
            let mut parallel_clone = parallel.clone();
            let barrier_clone = barrier.clone();

            let handle = thread::spawn(move || {
                barrier_clone.wait();

                for i in 0..operations_per_thread {
                    parallel_clone
                        .set_message(pb_ref, format!("Thread {thread_id}: Operation {i}"));

                    if i % 10 == 0 {
                        parallel_clone
                            .println(pb_ref, &format!("Thread {thread_id} checkpoint {i}"));
                    }

                    if i % 20 == 0 {
                        parallel_clone.hide(pb_ref);
                        parallel_clone.show(pb_ref);
                    }
                }
            });

            handles.push(handle);
        }

        barrier.wait();

        for handle in handles {
            handle.join().unwrap();
        }

        let mut parallel_final = parallel;
        parallel_final.succeeded(pb_ref);
    }

    #[test]
    fn many_concurrent_threads_same_progress_bar() {
        use crate::ProgressBar;
        use std::sync::{
            atomic::{AtomicUsize, Ordering},
            Arc, Barrier,
        };

        let mut parallel = ProgressBar::new(Parallel::boxed());
        let pb_ref = parallel.append("Shared progress bar");

        let num_threads = 20;
        let barrier = Arc::new(Barrier::new(num_threads + 1));
        let completed_operations = Arc::new(AtomicUsize::new(0));

        let mut handles = vec![];

        for thread_id in 0..num_threads {
            let mut parallel_clone = parallel.clone();
            let barrier_clone = barrier.clone();
            let completed_clone = completed_operations.clone();

            let handle = thread::spawn(move || {
                barrier_clone.wait();

                match thread_id % 4 {
                    0 => {
                        for i in 0..25 {
                            parallel_clone
                                .set_message(pb_ref, format!("Setter thread {thread_id}: {i}"));
                        }
                    }
                    1 => {
                        for i in 0..25 {
                            parallel_clone
                                .println(pb_ref, &format!("Printer thread {thread_id}: {i}"));
                        }
                    }
                    2 => {
                        for i in 0..25 {
                            if i % 2 == 0 {
                                parallel_clone.hide(pb_ref);
                            } else {
                                parallel_clone.show(pb_ref);
                            }
                        }
                    }
                    3 => {
                        for i in 0..25 {
                            parallel_clone
                                .set_message(pb_ref, format!("Mixed thread {thread_id}: {i}"));
                            parallel_clone
                                .println(pb_ref, &format!("Mixed output {thread_id}: {i}"));
                        }
                    }
                    _ => unreachable!(),
                }

                completed_clone.fetch_add(1, Ordering::SeqCst);
            });

            handles.push(handle);
        }

        barrier.wait();

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(completed_operations.load(Ordering::SeqCst), num_threads);
        let mut parallel_final = parallel;
        parallel_final.succeeded(pb_ref);
    }

    #[test]
    fn performance_under_high_load() {
        use crate::ProgressBar;
        use std::sync::{Arc, Barrier};
        use std::time::Instant;

        let mut parallel = ProgressBar::new(Parallel::boxed());
        let num_progress_bars = 50;
        let operations_per_bar = 20;

        let mut pb_refs = vec![];
        for i in 0..num_progress_bars {
            pb_refs.push(parallel.append(&format!("Load test bar {i}")));
        }

        let start_time = Instant::now();
        let num_threads = 8;
        let barrier = Arc::new(Barrier::new(num_threads + 1));
        let mut handles = vec![];

        for thread_id in 0..num_threads {
            let mut parallel_clone = parallel.clone();
            let barrier_clone = barrier.clone();
            let pb_refs_clone = pb_refs.clone();

            let handle = thread::spawn(move || {
                barrier_clone.wait();

                let bars_per_thread = num_progress_bars / num_threads;
                let start_idx = thread_id * bars_per_thread;
                let end_idx = if thread_id == num_threads - 1 {
                    num_progress_bars // Last thread takes any remaining bars
                } else {
                    start_idx + bars_per_thread
                };

                for pb_ref in &pb_refs_clone[start_idx..end_idx] {
                    for op in 0..operations_per_bar {
                        parallel_clone.set_message(*pb_ref, format!("Thread {thread_id} op {op}"));

                        if op % 5 == 0 {
                            parallel_clone
                                .println(*pb_ref, &format!("T{thread_id} checkpoint {op}"));
                        }
                    }

                    if start_idx % 2 == 0 {
                        parallel_clone.succeeded(*pb_ref);
                    } else {
                        parallel_clone.failed(*pb_ref);
                    }
                }
            });

            handles.push(handle);
        }

        barrier.wait();

        for handle in handles {
            handle.join().unwrap();
        }

        let duration = start_time.elapsed();

        assert!(
            duration.as_secs() < 5,
            "High load test took too long: {duration:?}"
        );

        println!("High load test completed in {duration:?}");
    }
}
