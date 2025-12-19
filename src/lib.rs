// Library exports for ecce package
// This allows integration tests and external crates to use ecce modules

pub mod config;
pub mod pattern;
pub mod watcher;
pub mod agent;
pub mod utils;

// Re-export commonly used types for convenience
pub use config::{Agent, Config, McpServer, Profile, Task};
pub use pattern::{EccePattern, PatternDetector, PatternType};
pub use watcher::FileWatcher;
