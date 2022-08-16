// client: HTTP client and associated methods
#![forbid(unsafe_code)]
#![forbid(missing_docs)]
use indicatif::{
    ProgressBarIter,
    ProgressDrawTarget,
    ProgressStyle,
};
use std::io::Write;

// How many times per second to redraw the progress bar.
const PROGRESS_UPDATE_HZ: u8 = 8;

const PROGRESS_CHARS: &str = "#>-";
const PROGRESS_FINISHED_MSG: &str = "done.";
const PROGRESS_TEMPLATE: &str = concat!(
    "{spinner:.green} ",
    "[{elapsed_precise}] ",
    "[{bar:40.cyan/blue}] ",
    "{bytes}/{total_bytes} ",
    "({eta})",
    " {msg}",
);

const PROGRESS_TEMPLATE_NO_COLOR: &str = concat!(
    "{spinner} ",
    "[{elapsed_precise}] ",
    "[{bar:40}] ",
    "{bytes}/{total_bytes} ",
    "({eta})",
    " {msg}",
);

#[derive(Default)]
pub struct ProgressBarBuilder {
    no_color: bool,
    quiet:    bool,
    size:     Option<u64>,
}

impl ProgressBarBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn no_color(mut self, no_color: bool) -> Self {
        self.no_color = no_color;
        self
    }

    pub fn quiet(mut self, quiet: bool) -> Self {
        self.quiet = quiet;
        self
    }

    pub fn size(mut self, size: Option<u64>) -> Self {
        self.size = size;
        self
    }

    pub fn build(self) -> ProgressBar {
        // No progress bar for quiet mode.
        if self.quiet {
            return ProgressBar {
                bar: indicatif::ProgressBar::hidden(),
            };
        }

        // We want to limit refreshes to once per second, so we have to make a
        // new draw target.
        let target = ProgressDrawTarget::stderr_with_hz(PROGRESS_UPDATE_HZ);

        let bar = if self.size.is_some() {
            // If we know the total size, setup a nice bar
            let template = if self.no_color {
                PROGRESS_TEMPLATE_NO_COLOR
            }
            else {
                PROGRESS_TEMPLATE
            };

            let style = ProgressStyle::default_bar()
                .template(template)
                .unwrap()
                .progress_chars(PROGRESS_CHARS);

            let pb = indicatif::ProgressBar::with_draw_target(
                self.size,
                target,
            );

            pb.set_style(style);

            pb
        }
        else {
            // Otherwise, just a simple spinner
            let pb = indicatif::ProgressBar::new_spinner();
            pb.set_draw_target(target);

            pb
        };

        ProgressBar {
            bar,
        }
    }
}

pub struct ProgressBar {
    bar: indicatif::ProgressBar,
}

impl ProgressBar {
    pub fn wrap_write<W: Write>(&self, writer: W) -> ProgressBarIter<W> {
        self.bar.wrap_write(writer)
    }

    pub fn finished(&self) {
        self.bar.finish_with_message(PROGRESS_FINISHED_MSG)
    }
}
