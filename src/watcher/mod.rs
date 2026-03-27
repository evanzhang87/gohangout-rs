//! File system monitoring for configuration hot reload

mod fs_watcher;

pub use fs_watcher::ConfigWatcher;
pub use fs_watcher::WatcherError;