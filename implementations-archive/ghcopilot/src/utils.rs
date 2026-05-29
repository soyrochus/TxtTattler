use std::sync::Arc;

use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use tracing_subscriber::EnvFilter;

use crate::domain::ports::{ProgressHandle, Reporter};

pub fn init_tracing(verbose: bool) {
    let filter = if verbose { "debug" } else { "info" };
    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(filter))
        .with_target(false)
        .without_time()
        .try_init();
}

#[derive(Clone)]
pub struct ConsoleReporter {
    verbose: bool,
}

impl ConsoleReporter {
    pub fn new(verbose: bool) -> Self {
        Self { verbose }
    }
}

impl Reporter for ConsoleReporter {
    fn status(&self, message: &str) {
        println!("{} {}", style("🎙").cyan(), style(message).bold());
    }

    fn detail(&self, message: &str) {
        if self.verbose {
            println!("{} {}", style("·").dim(), style(message).dim());
        }
    }

    fn success(&self, message: &str) {
        println!("{} {}", style("✨").green(), style(message).green().bold());
    }

    fn progress(&self, message: &str, len: u64) -> Box<dyn ProgressHandle> {
        let progress = ProgressBar::new(len);
        let style = ProgressStyle::with_template(
            "{spinner:.cyan} {msg} [{bar:30.magenta/blue}] {pos}/{len}",
        )
        .expect("progress template is valid")
        .progress_chars("=>-");
        progress.set_style(style);
        progress.set_message(message.to_string());
        Box::new(IndicatifProgressHandle {
            inner: Arc::new(progress),
        })
    }
}

struct IndicatifProgressHandle {
    inner: Arc<ProgressBar>,
}

impl ProgressHandle for IndicatifProgressHandle {
    fn inc(&self, delta: u64) {
        self.inner.inc(delta);
    }

    fn set_message(&self, message: &str) {
        self.inner.set_message(message.to_string());
    }

    fn finish_with_message(&self, message: &str) {
        self.inner.finish_with_message(message.to_string());
    }
}
