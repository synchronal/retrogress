use std::{thread, time};

use console::style;

fn main() {
    let mut progress = retrogress::ProgressBar::new(retrogress::Sync::boxed());

    for i in 1..10 {
        let pb = progress.append(&format!("running step {i}"));
        thread::sleep(time::Duration::from_millis(1000));
        if i % 3 == 0 {
            let msg = format!("{}", style("== Error: OH NO!").red());
            progress.println(pb, &msg);
            progress.failed(pb);
        } else {
            progress.succeeded(pb);
        }
    }

    let _ = progress.prompt("Here is a prompt for you to type something > ");
}
