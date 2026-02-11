use console::style;
use rand::RngExt;
use std::{thread, time};

fn main() {
    let mut progress = retrogress::ProgressBar::new(retrogress::Parallel::boxed());
    let mut handles = vec![];

    // Spawn multiple threads with progress bars
    for i in 1..=5 {
        let mut progress_clone = progress.clone();
        let pb = progress_clone.append(&format!("Thread {i} working"));

        let handle = thread::spawn(move || {
            let mut rng = rand::rng();
            // Simulate work with progress updates
            for j in 1..=20 {
                thread::sleep(time::Duration::from_millis(rng.random_range(0..10) * 50));
                progress_clone.set_message(pb, format!("Thread {i} - step {j}/20"));

                if i % 2 == 0 {
                    let msg = format!("{}", style(format!("Thread {i} did stuff")).yellow());
                    progress_clone.println(pb, &msg);
                }
            }

            if i == 2 {
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

    let input = progress.prompt("Here is a prompt for you to type something > ");
    drop(progress);
    match input {
        Ok(msg) => println!("Received input: {msg}"),
        Err(e) => eprintln!("Error reading input: {e}"),
    }
}
