use crate::progress::Ref;
use crate::render::ProgressBar;
use crate::Progress;

use console::style;
use std::collections::HashMap;

/// An implementation of `Progress` intended for usages when one
/// progress bar runs at a time.
///
/// ```rust
/// use retrogress::{ProgressBar, Sync};
/// let mut progress = ProgressBar::new(Sync::boxed());
/// ```
pub struct Sync {
    bars: HashMap<Ref, ProgressBar>,
}

impl Sync {
    pub fn new() -> Self {
        console::set_colors_enabled(true);
        console::set_colors_enabled_stderr(true);

        Self {
            bars: HashMap::new(),
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
        let pb = ProgressBar::new(msg.to_string());
        let reference = Ref::new();
        self.bars.insert(reference, pb);
        reference
    }

    fn failed(&mut self, reference: Ref) {
        let pb = self.bars.get(&reference).unwrap();
        pb.set_prefix(format!("{}", style("𝗑").bold().bright().red()));
        pb.finish();
    }

    fn hide(&mut self, reference: Ref) {
        let pb = self.bars.get(&reference).unwrap();
        pb.hide();
    }

    fn println(&mut self, reference: Ref, msg: &str) {
        let pb = self.bars.get(&reference).unwrap();
        pb.println(msg);
    }

    fn set_message(&mut self, reference: Ref, msg: String) {
        let pb = self.bars.get(&reference).unwrap();
        pb.set_message(msg);
    }

    fn show(&mut self, reference: Ref) {
        let pb = self.bars.get(&reference).unwrap();
        pb.show();
    }

    fn succeeded(&mut self, reference: Ref) {
        let pb = self.bars.get(&reference).unwrap();
        pb.set_prefix(format!("{}", style("✓").bold().green()));
        pb.finish();
    }
}
