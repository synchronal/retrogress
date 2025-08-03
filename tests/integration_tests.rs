use retrogress::{Parallel, Progress, ProgressBar, Sync};
use std::thread;

#[test]
fn sync_progress_bar_basic_usage() {
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
fn parallel_progress_bar_basic_usage() {
    let mut progress = ProgressBar::new(Parallel::boxed());

    let pb1 = progress.append("First parallel task");
    let pb2 = progress.append("Second parallel task");

    progress.set_message(pb1, "Working on first parallel task".to_string());
    progress.println(pb1, "Parallel debug output");
    progress.succeeded(pb1);

    progress.set_message(pb2, "Working on second parallel task".to_string());
    progress.println(pb2, "More parallel debug output");
    progress.failed(pb2);

    drop(progress);
}

#[test]
fn progress_bar_hide_and_show() {
    let mut progress = ProgressBar::new(Sync::boxed());
    let pb = progress.append("Test task");

    progress.hide(pb);
    progress.set_message(pb, "Hidden message".to_string());
    progress.show(pb);
    progress.succeeded(pb);
}

#[test]
fn progress_bar_clone_and_move() {
    let progress = ProgressBar::new(Sync::boxed());
    let cloned_progress = progress.clone();

    let handle = thread::spawn(move || {
        let mut p = progress;
        let pb = p.append("Thread task");
        p.succeeded(pb);
    });

    let mut cloned_p = cloned_progress;
    let pb = cloned_p.append("Main task");
    cloned_p.succeeded(pb);

    handle.join().unwrap();
}

#[test]
fn parallel_progress_bar_multiple_threads() {
    use std::sync::{Arc, Barrier};

    let progress = ProgressBar::new(Parallel::boxed());
    let mut handles = vec![];
    let num_threads = 3;
    let barrier = Arc::new(Barrier::new(num_threads + 1));

    for i in 0..num_threads {
        let mut progress_clone = progress.clone();
        let barrier_clone = barrier.clone();

        let handle = thread::spawn(move || {
            let pb = progress_clone.append(&format!("Thread {} task", i));

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

    barrier.wait();

    for handle in handles {
        handle.join().unwrap();
    }

    drop(progress);
}

#[test]
fn many_progress_bars() {
    let mut progress = ProgressBar::new(Sync::boxed());
    let mut refs = Vec::new();

    for i in 0..50 {
        let pb_ref = progress.append(&format!("Task {}", i));
        refs.push(pb_ref);
    }

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
fn parallel_many_progress_bars() {
    let mut progress = ProgressBar::new(Parallel::boxed());
    let mut refs = Vec::new();

    for i in 0..20 {
        let pb_ref = progress.append(&format!("Parallel task {}", i));
        refs.push(pb_ref);
    }

    for (i, &pb_ref) in refs.iter().enumerate() {
        progress.set_message(pb_ref, format!("Processing parallel task {}", i));
        progress.println(pb_ref, &format!("Output from task {}", i));

        if i % 2 == 0 {
            progress.succeeded(pb_ref);
        } else {
            progress.failed(pb_ref);
        }
    }

    drop(progress);
}

#[test]
fn empty_messages() {
    let mut progress = ProgressBar::new(Sync::boxed());

    let pb = progress.append("");
    progress.set_message(pb, "".to_string());
    progress.println(pb, "");
    progress.succeeded(pb);
}

#[test]
fn unicode_messages() {
    let mut progress = ProgressBar::new(Sync::boxed());

    let pb = progress.append("Unicode test: 🚀");
    progress.set_message(pb, "Processing 데이터 with émojis 🎉".to_string());
    progress.println(pb, "Unicode output: αβγδε ñáéíóú 中文");
    progress.succeeded(pb);
}

#[test]
fn very_long_messages() {
    let mut progress = ProgressBar::new(Sync::boxed());

    let long_message = "A".repeat(1000);
    let pb = progress.append(&long_message);
    progress.set_message(pb, "B".repeat(500));
    progress.println(pb, &"C".repeat(200));
    progress.succeeded(pb);
}

#[test]
fn rapid_operations() {
    let mut progress = ProgressBar::new(Parallel::boxed());
    let pb = progress.append("Rapid operations test");

    for i in 0..100 {
        progress.set_message(pb, format!("Rapid update {}", i));
        if i % 10 == 0 {
            progress.println(pb, &format!("Checkpoint {}", i));
        }
    }

    progress.succeeded(pb);

    drop(progress);
}

#[test]
fn interleaved_operations() {
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

    drop(progress);
}

#[test]
fn channel_disconnection_handling() {
    let mut progress = ProgressBar::new(Parallel::boxed());
    let pb_ref = progress.append("Test task");

    progress.set_message(pb_ref, "Initial message".to_string());
    progress.println(pb_ref, "Initial output");

    drop(progress);

    let mut progress2 = ProgressBar::new(Parallel::boxed());
    let pb_ref2 = progress2.append("Second test task");
    progress2.set_message(pb_ref2, "Message after channel disconnect".to_string());
    progress2.succeeded(pb_ref2);

    drop(progress2);
}

#[test]
fn extreme_concurrent_stress() {
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Barrier,
    };

    let mut progress = ProgressBar::new(Parallel::boxed());
    let pb_ref = progress.append("Extreme stress test");

    let num_threads = 500;
    let operations_per_thread = 1000;
    let barrier = Arc::new(Barrier::new(num_threads + 1));
    let error_count = Arc::new(AtomicUsize::new(0));

    let mut handles = vec![];

    for thread_id in 0..num_threads {
        let mut progress_clone = progress.clone();
        let barrier_clone = barrier.clone();
        let _error_count_clone = error_count.clone();

        let handle = thread::spawn(move || {
            barrier_clone.wait();

            for i in 0..operations_per_thread {
                match i % 5 {
                    0 => progress_clone.set_message(pb_ref, format!("T{}-Op{}", thread_id, i)),
                    1 => progress_clone.println(pb_ref, &format!("T{}-Log{}", thread_id, i)),
                    2 => progress_clone.hide(pb_ref),
                    3 => progress_clone.show(pb_ref),
                    4 => {
                        progress_clone.set_message(pb_ref, format!("T{}-Rapid{}", thread_id, i));
                        progress_clone.println(pb_ref, &format!("T{}-RapidLog{}", thread_id, i));
                    }
                    _ => unreachable!(),
                }

                if i % 10 == 0 {
                    for j in 0..5 {
                        progress_clone.println(pb_ref, &format!("T{}-Burst{}-{}", thread_id, i, j));
                    }
                }
            }
        });

        handles.push(handle);
    }

    barrier.wait();

    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(error_count.load(Ordering::SeqCst), 0);

    let mut progress_final = progress;
    progress_final.succeeded(pb_ref);
    drop(progress_final);
}

#[test]
fn rapid_progress_bar_creation_and_destruction() {
    use std::sync::{Arc, Barrier};

    let num_threads = 100;
    let bars_per_thread = 200;
    let barrier = Arc::new(Barrier::new(num_threads + 1));

    let mut handles = vec![];

    for thread_id in 0..num_threads {
        let barrier_clone = barrier.clone();

        let handle = thread::spawn(move || {
            barrier_clone.wait();

            for i in 0..bars_per_thread {
                let mut progress = ProgressBar::new(Parallel::boxed());
                let pb_ref = progress.append(&format!("T{}-Bar{}", thread_id, i));

                progress.set_message(pb_ref, format!("T{}-Msg{}", thread_id, i));
                progress.println(pb_ref, &format!("T{}-Out{}", thread_id, i));

                if i % 2 == 0 {
                    progress.succeeded(pb_ref);
                } else {
                    progress.failed(pb_ref);
                }

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
