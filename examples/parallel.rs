use std::{thread, time};

use console::style;

fn main() {
    let progress = retrogress::ProgressBar::new(retrogress::Parallel::boxed());
    let mut handles = vec![];

    // Spawn multiple threads with progress bars
    for i in 1..=5 {
        let mut progress_clone = progress.clone();
        let pb = progress_clone.append(&format!("Thread {i} working"));

        let handle = thread::spawn(move || {
            // Simulate work with progress updates
            for j in 1..=5 {
                thread::sleep(time::Duration::from_millis(500));
                progress_clone.set_message(pb, format!("Thread {i} - step {j}/5"));

                if j == 3 && i % 2 == 0 {
                    let msg = format!(
                        "{}",
                        style(format!("Thread {i} encountered an issue")).yellow()
                    );
                    progress_clone.println(pb, &msg);
                }
            }

            // Some threads succeed, some fail
            if i % 3 == 0 {
                progress_clone.failed(pb);
            } else {
                progress_clone.succeeded(pb);
            }
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }
}
