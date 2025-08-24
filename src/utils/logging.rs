//! Logging utilities for netbeat.

/// Custom logger for netbeat.
#[derive(Debug, Clone)]
pub struct Logger {
    pub verbose: bool,
    pub quiet: bool,
}

impl Logger {
    /// Creates a new logger instance.
    pub fn new(verbose: bool, quiet: bool) -> Self {
        Self { verbose, quiet }
    }

    /// Final results - goes to STDOUT
    pub fn result(&self, msg: &str) {
        println!("{msg}");
    }

    /// Progress/status - goes to STDERR, suppressible by --quiet
    pub fn info(&self, msg: &str) {
        if !self.quiet {
            eprintln!("{msg}");
        }
    }

    /// Detailed debug info - goes to STDERR, only shown with --verbose
    pub fn verbose(&self, msg: &str) {
        if self.verbose && !self.quiet {
            eprintln!("🔍 [VERBOSE] {msg}");
        }
    }

    /// Errors - ALWAYS go to STDERR (even with --quiet)
    pub fn error(&self, msg: &str) {
        eprintln!("❌ {msg}");
    }

    /// Warnings - go to STDERR, suppressible by --quiet
    pub fn warn(&self, msg: &str) {
        if !self.quiet {
            eprintln!("⚠️  {msg}");
        }
    }

    /// Success messages - go to STDERR, suppressible by --quiet
    pub fn success(&self, msg: &str) {
        if !self.quiet {
            eprintln!("✅ {msg}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Basic smoke test... defer testing to cli itself.
    #[test]
    fn test_logger() {
        let logger = Logger::new(true, false);
        logger.result("Result");
        logger.info("Info");
        logger.verbose("Verbose");
        logger.error("Error");
        logger.warn("Warning");
        logger.success("Success");
    }
}
