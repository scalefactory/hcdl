// Client configuration

/// [`ClientConfig`] is a configuration for [`Client`].
#[derive(Debug, Default)]
pub struct ClientConfig {
    /// Control the output of colour in the crate messages and progress bars.
    pub no_color: bool,

    /// Controls the output of text in the crate.
    pub quiet: bool,
}

impl ClientConfig {
    /// Create a new [`ClientConfig`] with the default settings.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// `no_color` controls the output of colours in the various output of the
    /// crate.
    #[must_use]
    pub fn no_color(mut self, no_color: bool) -> Self {
        self.no_color = no_color;
        self
    }

    /// `quiet` controls the various text output of the crate.
    #[must_use]
    pub fn quiet(mut self, quiet: bool) -> Self {
        self.quiet = quiet;
        self
    }
}
