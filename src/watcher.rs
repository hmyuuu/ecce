use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use std::thread;
use std::time::Duration;

use crate::pattern::{EccePattern, PatternDetector};

pub struct FileWatcher {
    last_content: String,
    detector: PatternDetector,
    poll_interval: Duration,
}

impl FileWatcher {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        Self::with_interval(path, 500)
    }

    pub fn with_interval<P: AsRef<Path>>(path: P, interval_ms: u64) -> Result<Self> {
        let initial_content =
            fs::read_to_string(&path).context("Failed to read initial file content")?;

        Ok(Self {
            last_content: initial_content,
            detector: PatternDetector::new(),
            poll_interval: Duration::from_millis(interval_ms),
        })
    }

    pub fn watch<P: AsRef<Path>>(&mut self, _path: P) -> Result<()> {
        // No-op: we don't need to set up watching, we'll poll directly
        Ok(())
    }

    /// Wait for file changes and return new patterns found
    pub fn wait_for_changes<P: AsRef<Path>>(&mut self, path: P) -> Result<Vec<EccePattern>> {
        loop {
            // Sleep for the poll interval
            thread::sleep(self.poll_interval);

            // Check for new patterns
            if let Some(patterns) = self.check_for_new_patterns(&path)? {
                if !patterns.is_empty() {
                    return Ok(patterns);
                }
            }
        }
    }

    /// Check for new patterns in file
    fn check_for_new_patterns<P: AsRef<Path>>(
        &mut self,
        path: P,
    ) -> Result<Option<Vec<EccePattern>>> {
        let current_content = fs::read_to_string(&path).context("Failed to read file content")?;

        // If content is identical, skip
        if current_content == self.last_content {
            return Ok(None);
        }

        // Check the entire file for new patterns
        let patterns = self.detector.detect_new_patterns(&current_content);

        // Update last content
        self.last_content = current_content;

        if patterns.is_empty() {
            Ok(None)
        } else {
            Ok(Some(patterns))
        }
    }

    /// Mark a pattern as processed
    pub fn mark_processed(&mut self, content: &str) {
        self.detector.mark_processed(content);
    }

    /// Update the watcher's content to match the current file
    pub fn update_content<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let current_content = fs::read_to_string(&path).context("Failed to read file content")?;
        self.last_content = current_content;
        Ok(())
    }

    /// Get the current file content
    pub fn current_content(&self) -> &str {
        &self.last_content
    }
}

