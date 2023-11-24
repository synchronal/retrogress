use crate::progress::Ref;
use crate::Progress;

use console::style;
use indicatif::ProgressBar;
use indicatif::ProgressStyle;
use std::collections::HashMap;
use std::time::Duration;

/// An implementation of `Progress` intended for usages when one
/// progress bar runs at a time.
///
/// ```rust
/// use retrogress::{ProgressBar, Sync};
/// let mut progress = ProgressBar::new(Sync::boxed());
/// ```

pub struct Sync {
    bars: HashMap<Ref, ProgressBar>,
    _running: ProgressStyle,
    _stopped: ProgressStyle,
}

impl Sync {
    pub fn new() -> Self {
        let spinner_style = ProgressStyle::with_template("{prefix:.bold} {spinner} {msg}")
            .unwrap()
            .tick_chars("⣾⣽⣻⢿⡿⣟⣯⣷");

        let stopped_style = ProgressStyle::with_template("{prefix:.bold} {msg}").unwrap();

        Self {
            bars: HashMap::new(),
            _running: spinner_style,
            _stopped: stopped_style,
        }
    }

    pub fn boxed() -> Box<Self> {
        Box::new(Self::new())
    }
}

impl Default for Sync {
    fn default() -> Self {
        Self::new()
    }
}

impl Progress for Sync {
    fn append(&mut self, msg: &str) -> Ref {
        let pb = ProgressBar::new(100).with_message(msg.to_string());
        pb.set_style(self._running.clone());
        pb.set_prefix(format!("{}", style("•").green()));
        pb.enable_steady_tick(Duration::from_millis(50));

        let reference = Ref::new();
        self.bars.insert(reference, pb);
        reference
    }
    fn failed(&mut self, reference: Ref) {
        let pb = self.bars.get(&reference).unwrap();
        pb.set_style(self._stopped.clone());
        pb.set_prefix(format!("{}", style("𝗑").bold().bright().red()));
        pb.finish();
    }

    fn println(&mut self, reference: Ref, msg: &str) {
        let pb = self.bars.get(&reference).unwrap();
        pb.println(msg);
    }

    fn set_message(&mut self, reference: Ref, msg: String) {
        let pb = self.bars.get(&reference).unwrap();
        pb.set_message(msg.clone());
    }

    fn succeeded(&mut self, reference: Ref) {
        let pb = self.bars.get(&reference).unwrap();
        pb.set_style(self._stopped.clone());
        pb.set_prefix(format!("{}", style("✓").bold().green()));
        pb.finish();
    }
}
