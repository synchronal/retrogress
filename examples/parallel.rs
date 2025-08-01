use std::{thread, time};

use console::style;
use retrogress::Progress;

fn main() {
    let progress = retrogress::ProgressBar::new(retrogress::Parallel::boxed());
    let mut handles = vec![];

    // Spawn multiple threads with progress bars
    for i in 1..=5 {
        let mut progress_clone = progress.clone();
        let handle = thread::spawn(move || {
            let pb = progress_clone.append(&format!("Thread {} working", i));

            // Simulate work with progress updates
            for j in 1..=5 {
                thread::sleep(time::Duration::from_millis(500));
                progress_clone.set_message(pb, format!("Thread {} - step {}/5", i, j));

                if j == 3 && i % 2 == 0 {
                    let msg = format!(
                        "{}",
                        style(format!("Thread {} encountered an issue", i)).yellow()
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
