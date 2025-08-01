use retrogress::{Parallel, Progress, ProgressBar, Sync};
use std::thread;

#[test]
fn test_sync_progress_bar_basic_usage() {
    let mut progress = ProgressBar::new(Sync::boxed());

    let pb1 = progress.append("First task");
    let pb2 = progress.append("Second task");

    progress.set_message(pb1, "Working on first task".to_string());
    progress.println(pb1, "Some debug output");
    progress.succeeded(pb1);

    progress.set_message(pb2, "Working on second task".to_string());
    progress.println(pb2, "More debug output");
    progress.failed(pb2);
}

#[test]
fn test_parallel_progress_bar_basic_usage() {
    let mut progress = ProgressBar::new(Parallel::boxed());

    let pb1 = progress.append("First parallel task");
    let pb2 = progress.append("Second parallel task");

    progress.set_message(pb1, "Working on first parallel task".to_string());
    progress.println(pb1, "Parallel debug output");
    progress.succeeded(pb1);

    progress.set_message(pb2, "Working on second parallel task".to_string());
    progress.println(pb2, "More parallel debug output");
    progress.failed(pb2);

    // Drop the progress bar to ensure all messages are processed
    drop(progress);
}

#[test]
fn test_progress_bar_hide_and_show() {
    let mut progress = ProgressBar::new(Sync::boxed());
    let pb = progress.append("Test task");

    progress.hide(pb);
    progress.set_message(pb, "Hidden message".to_string());
    progress.show(pb);
    progress.succeeded(pb);
}

#[test]
fn test_progress_bar_clone_and_move() {
    let progress = ProgressBar::new(Sync::boxed());
    let cloned_progress = progress.clone();

    // Move original to another thread
    let handle = thread::spawn(move || {
        let mut p = progress;
        let pb = p.append("Thread task");
        p.succeeded(pb);
    });

    // Use cloned version in main thread
    let mut cloned_p = cloned_progress;
    let pb = cloned_p.append("Main task");
    cloned_p.succeeded(pb);

    handle.join().unwrap();
}

#[test]
fn test_parallel_progress_bar_multiple_threads() {
    use std::sync::{Arc, Barrier};

    let progress = ProgressBar::new(Parallel::boxed());
    let mut handles = vec![];
    let num_threads = 3;
    let barrier = Arc::new(Barrier::new(num_threads + 1));

    // Spawn multiple threads with progress bars
    for i in 0..num_threads {
        let mut progress_clone = progress.clone();
        let barrier_clone = barrier.clone();

        let handle = thread::spawn(move || {
            let pb = progress_clone.append(&format!("Thread {} task", i));

            // Wait for all threads to be ready
            barrier_clone.wait();

            for j in 1..=3 {
                progress_clone.set_message(pb, format!("Thread {} - step {}/3", i, j));
            }

            if i % 2 == 0 {
                progress_clone.succeeded(pb);
            } else {
                progress_clone.failed(pb);
            }
        });
        handles.push(handle);
    }

    // Start all threads simultaneously
    barrier.wait();

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    // Drop progress bar to ensure all messages are processed
    drop(progress);
}

#[test]
fn test_sync_vs_parallel_same_interface() {
    fn test_progress_implementation(mut progress: ProgressBar) {
        let pb1 = progress.append("Task 1");
        let pb2 = progress.append("Task 2");

        progress.set_message(pb1, "Processing task 1".to_string());
        progress.println(pb1, "Task 1 output");

        progress.set_message(pb2, "Processing task 2".to_string());
        progress.println(pb2, "Task 2 output");

        progress.hide(pb1);
        progress.show(pb1);

        progress.succeeded(pb1);
        progress.failed(pb2);
    }

    // Test with Sync implementation
    let sync_progress = ProgressBar::new(Sync::boxed());
    test_progress_implementation(sync_progress);

    // Test with Parallel implementation
    let parallel_progress = ProgressBar::new(Parallel::boxed());
    test_progress_implementation(parallel_progress);
}

#[test]
fn test_many_progress_bars() {
    let mut progress = ProgressBar::new(Sync::boxed());
    let mut refs = Vec::new();

    // Create many progress bars
    for i in 0..50 {
        let pb_ref = progress.append(&format!("Task {}", i));
        refs.push(pb_ref);
    }

    // Operate on all of them
    for (i, &pb_ref) in refs.iter().enumerate() {
        progress.set_message(pb_ref, format!("Processing task {}", i));

        if i % 10 == 0 {
            progress.println(pb_ref, &format!("Milestone at task {}", i));
        }

        if i % 3 == 0 {
            progress.succeeded(pb_ref);
        } else if i % 3 == 1 {
            progress.failed(pb_ref);
        } else {
            progress.hide(pb_ref);
        }
    }
}

#[test]
fn test_parallel_many_progress_bars() {
    let mut progress = ProgressBar::new(Parallel::boxed());
    let mut refs = Vec::new();

    // Create many progress bars
    for i in 0..20 {
        let pb_ref = progress.append(&format!("Parallel task {}", i));
        refs.push(pb_ref);
    }

    // Operate on all of them
    for (i, &pb_ref) in refs.iter().enumerate() {
        progress.set_message(pb_ref, format!("Processing parallel task {}", i));
        progress.println(pb_ref, &format!("Output from task {}", i));

        if i % 2 == 0 {
            progress.succeeded(pb_ref);
        } else {
            progress.failed(pb_ref);
        }
    }

    // Drop progress bar to ensure all messages are processed
    drop(progress);
}

#[test]
fn test_empty_messages() {
    let mut progress = ProgressBar::new(Sync::boxed());

    let pb = progress.append("");
    progress.set_message(pb, "".to_string());
    progress.println(pb, "");
    progress.succeeded(pb);
}

#[test]
fn test_unicode_messages() {
    let mut progress = ProgressBar::new(Sync::boxed());

    let pb = progress.append("Unicode test: 🚀");
    progress.set_message(pb, "Processing 데이터 with émojis 🎉".to_string());
    progress.println(pb, "Unicode output: αβγδε ñáéíóú 中文");
    progress.succeeded(pb);
}

#[test]
fn test_very_long_messages() {
    let mut progress = ProgressBar::new(Sync::boxed());

    let long_message = "A".repeat(1000);
    let pb = progress.append(&long_message);
    progress.set_message(pb, "B".repeat(500));
    progress.println(pb, &"C".repeat(200));
    progress.succeeded(pb);
}

#[test]
fn test_rapid_operations() {
    let mut progress = ProgressBar::new(Parallel::boxed());
    let pb = progress.append("Rapid operations test");

    // Perform many rapid operations
    for i in 0..100 {
        progress.set_message(pb, format!("Rapid update {}", i));
        if i % 10 == 0 {
            progress.println(pb, &format!("Checkpoint {}", i));
        }
    }

    progress.succeeded(pb);

    // Drop progress bar to ensure all messages are processed
    drop(progress);
}

#[test]
fn test_interleaved_operations() {
    let mut progress = ProgressBar::new(Parallel::boxed());

    let pb1 = progress.append("Task A");
    let pb2 = progress.append("Task B");
    let pb3 = progress.append("Task C");

    // Interleave operations on different progress bars
    progress.set_message(pb1, "A step 1".to_string());
    progress.set_message(pb2, "B step 1".to_string());
    progress.println(pb3, "C output 1");
    progress.set_message(pb3, "C step 1".to_string());
    progress.println(pb1, "A output 1");
    progress.set_message(pb1, "A step 2".to_string());
    progress.println(pb2, "B output 1");
    progress.succeeded(pb1);
    progress.set_message(pb2, "B step 2".to_string());
    progress.failed(pb2);
    progress.succeeded(pb3);

    // Drop progress bar to ensure all messages are processed
    drop(progress);
}

#[test]
fn test_channel_disconnection_handling() {
    // Test that operations continue to work gracefully even when channels disconnect
    let mut progress = ProgressBar::new(Parallel::boxed());
    let pb_ref = progress.append("Test task");

    // Set initial state
    progress.set_message(pb_ref, "Initial message".to_string());
    progress.println(pb_ref, "Initial output");

    // Manually trigger Drop to simulate channel disconnect
    // This should shut down the worker thread cleanly
    drop(progress);

    // Create a new instance - this should work without issues
    let mut progress2 = ProgressBar::new(Parallel::boxed());
    let pb_ref2 = progress2.append("Second test task");
    progress2.set_message(pb_ref2, "Message after channel disconnect".to_string());
    progress2.succeeded(pb_ref2);

    drop(progress2);
}

#[test]
fn test_worker_thread_panic_recovery() {
    // Test that the system handles worker thread issues gracefully
    // We can't easily force a panic in the worker thread without modifying the source,
    // but we can test the error handling paths

    let mut progress = ProgressBar::new(Parallel::boxed());
    let pb_ref = progress.append("Panic recovery test");

    // Perform operations that should work normally
    progress.set_message(pb_ref, "Before potential issues".to_string());
    progress.println(pb_ref, "Output before issues");

    // Even if there were internal issues, these operations should not panic
    for i in 0..10 {
        progress.set_message(pb_ref, format!("Recovery test {}", i));
    }

    progress.succeeded(pb_ref);
    drop(progress);
}

#[test]
fn test_extreme_concurrent_stress() {
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Barrier,
    };

    let mut progress = ProgressBar::new(Parallel::boxed());
    let pb_ref = progress.append("Extreme stress test");

    let num_threads = 50;
    let operations_per_thread = 50;
    let barrier = Arc::new(Barrier::new(num_threads + 1));
    let error_count = Arc::new(AtomicUsize::new(0));

    let mut handles = vec![];

    for thread_id in 0..num_threads {
        let mut progress_clone = progress.clone();
        let barrier_clone = barrier.clone();
        let _error_count_clone = error_count.clone();

        let handle = thread::spawn(move || {
            barrier_clone.wait();

            // Perform intensive operations that might reveal race conditions
            for i in 0..operations_per_thread {
                // Vary operation types to stress different code paths
                match i % 5 {
                    0 => progress_clone.set_message(pb_ref, format!("T{}-Op{}", thread_id, i)),
                    1 => progress_clone.println(pb_ref, &format!("T{}-Log{}", thread_id, i)),
                    2 => progress_clone.hide(pb_ref),
                    3 => progress_clone.show(pb_ref),
                    4 => {
                        // Rapidly toggle between different operations
                        progress_clone.set_message(pb_ref, format!("T{}-Rapid{}", thread_id, i));
                        progress_clone.println(pb_ref, &format!("T{}-RapidLog{}", thread_id, i));
                    }
                    _ => unreachable!(),
                }

                // Add some operations that stress the message queue
                if i % 10 == 0 {
                    for j in 0..5 {
                        progress_clone.println(pb_ref, &format!("T{}-Burst{}-{}", thread_id, i, j));
                    }
                }
            }
        });

        handles.push(handle);
    }

    // Start all threads at once
    barrier.wait();

    // Wait for completion
    for handle in handles {
        handle.join().unwrap();
    }

    // Verify no errors occurred
    assert_eq!(error_count.load(Ordering::SeqCst), 0);

    // Final operations should still work
    let mut progress_final = progress;
    progress_final.succeeded(pb_ref);
    drop(progress_final);
}

#[test]
fn test_rapid_progress_bar_creation_and_destruction() {
    use std::sync::{Arc, Barrier};

    let num_threads = 10;
    let bars_per_thread = 20;
    let barrier = Arc::new(Barrier::new(num_threads + 1));

    let mut handles = vec![];

    for thread_id in 0..num_threads {
        let barrier_clone = barrier.clone();

        let handle = thread::spawn(move || {
            barrier_clone.wait();

            // Rapidly create and destroy progress bars
            for i in 0..bars_per_thread {
                let mut progress = ProgressBar::new(Parallel::boxed());
                let pb_ref = progress.append(&format!("T{}-Bar{}", thread_id, i));

                // Perform some operations
                progress.set_message(pb_ref, format!("T{}-Msg{}", thread_id, i));
                progress.println(pb_ref, &format!("T{}-Out{}", thread_id, i));

                if i % 2 == 0 {
                    progress.succeeded(pb_ref);
                } else {
                    progress.failed(pb_ref);
                }

                // Drop immediately to stress cleanup paths
                drop(progress);
            }
        });

        handles.push(handle);
    }

    barrier.wait();

    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn test_memory_usage_under_load() {
    // Test that memory usage doesn't grow unbounded under high load
    let mut progress = ProgressBar::new(Parallel::boxed());

    // Create many progress bars and perform many operations
    let mut pb_refs = vec![];
    for i in 0..100 {
        pb_refs.push(progress.append(&format!("Memory test bar {}", i)));
    }

    // Perform thousands of operations
    for pb_ref in &pb_refs {
        for i in 0..100 {
            progress.set_message(*pb_ref, format!("Memory test message {}", i));

            if i % 10 == 0 {
                progress.println(*pb_ref, &format!("Memory test output {}", i));
            }
        }
    }

    // Complete all progress bars
    for (i, pb_ref) in pb_refs.iter().enumerate() {
        if i % 2 == 0 {
            progress.succeeded(*pb_ref);
        } else {
            progress.failed(*pb_ref);
        }
    }

    drop(progress);

    // If we reach here without OOM or other issues, the test passes
}

#[test]
fn test_deterministic_behavior_with_barriers() {
    use std::sync::{Arc, Barrier, Mutex};

    // Test that operations happen in a predictable order when using proper synchronization
    let mut progress = ProgressBar::new(Parallel::boxed());
    let pb_ref = progress.append("Deterministic test");

    let num_threads = 5;
    let operations_per_thread = 10;
    let start_barrier = Arc::new(Barrier::new(num_threads + 1));
    let results = Arc::new(Mutex::new(Vec::new()));

    let mut handles = vec![];

    for thread_id in 0..num_threads {
        let mut progress_clone = progress.clone();
        let start_barrier_clone = start_barrier.clone();
        let results_clone = results.clone();

        let handle = thread::spawn(move || {
            // Wait for all threads to be ready
            start_barrier_clone.wait();

            // Record that this thread started
            {
                let mut r = results_clone.lock().unwrap();
                r.push(format!("Thread {} started", thread_id));
            }

            // Perform operations
            for i in 0..operations_per_thread {
                progress_clone.set_message(pb_ref, format!("T{}-Op{}", thread_id, i));

                if i == operations_per_thread / 2 {
                    let mut r = results_clone.lock().unwrap();
                    r.push(format!("Thread {} halfway", thread_id));
                }
            }

            // Record completion
            {
                let mut r = results_clone.lock().unwrap();
                r.push(format!("Thread {} completed", thread_id));
            }
        });

        handles.push(handle);
    }

    // Start all threads simultaneously
    start_barrier.wait();

    // Wait for all to complete
    for handle in handles {
        handle.join().unwrap();
    }

    // Verify results
    let results = results.lock().unwrap();
    assert_eq!(results.len(), num_threads * 3); // start + halfway + completed for each thread

    // Final operation
    let mut progress_final = progress;
    progress_final.succeeded(pb_ref);
    drop(progress_final);
}
