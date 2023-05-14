use std::path::{Path, PathBuf};

/// Provides information about dependencies between packages.
pub trait DepsInfo {
    /// Returns a list of runtime dependencies of the given path.
    fn get_runtime_deps(&self, path: &Path) -> Result<Vec<PathBuf>, String>;
}
