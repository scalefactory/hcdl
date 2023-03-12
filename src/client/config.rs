// Client configuration
#[derive(Debug, Default)]
pub struct ClientConfig {
    pub no_color: bool,
    pub quiet: bool,
}

impl ClientConfig {
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
}
