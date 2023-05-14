use std::{
    path::{Path, PathBuf},
    process::Command,
};

use super::api::DepsInfo;

/// Invokes the `nix` tool to provide dependency information.
/// This will be used until Rix doesn't implement the entirety of Nix.
#[derive(Default)]
pub struct NixDelegationStore {}

impl DepsInfo for NixDelegationStore {
    fn get_runtime_deps(&self, path: &Path) -> Result<Vec<PathBuf>, String> {
        let path_as_str = path
            .to_str()
            .ok_or_else(|| format!("Failed to convert {path:?} to string,"))?;
        let nix_args = vec!["--query", "--requisites", path_as_str];
        let _show_drv_out = Command::new("nix-store")
            .args(&nix_args)
            .output()
            .map_err(|err| {
                format!("Failed to execute `nix-store` with args {nix_args:?}. Error: {err}.")
            })?;

        if !_show_drv_out.status.success() {
            return Err(format!(
                "Failed to get runtime dependencies of '{path_as_str}'. Error: {}.",
                std::str::from_utf8(&_show_drv_out.stderr).unwrap_or("<failed to decode UTF-8>")
            ));
        }

        Ok(std::str::from_utf8(&_show_drv_out.stdout)
            .map_err(|err| {
                format!("Failed to decode the output of nix-store with UTF-8. Error: {err}")
            })?
            .lines()
            .map(PathBuf::from)
            .collect())
    }
}
