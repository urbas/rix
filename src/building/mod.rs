use crate::derivations::{load_derivation, Derivation};
use crate::sandbox;
use crate::store::api::DepsInfo;
use std::collections::HashSet;
use std::fs::File;
use std::os::unix::io::{AsRawFd, FromRawFd, RawFd};
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub struct BuildConfig<'a> {
    build_dir: &'a Path,
    deps_info: &'a dyn DepsInfo,
    derivation: &'a Derivation,
    stderr: Option<&'a File>,
    stdout: Option<&'a File>,
}

impl<'a> BuildConfig<'a> {
    pub fn new(
        derivation: &'a Derivation,
        build_dir: &'a Path,
        deps_info: &'a dyn DepsInfo,
    ) -> BuildConfig<'a> {
        BuildConfig {
            build_dir,
            deps_info,
            derivation,
            stderr: None,
            stdout: None,
        }
    }

    pub fn stdout_to_file(&mut self, file: &'a File) {
        self.stdout = Some(file);
    }

    pub fn stderr_to_file(&mut self, file: &'a File) {
        self.stderr = Some(file);
    }
}

pub fn build_derivation_sandboxed(config: &BuildConfig) -> Result<i32, String> {
    // this function assumes all derivation inputs are present and won't be
    // GC'd for the duration of this build
    let stdout_fd = config.stdout.map(|file| file.as_raw_fd());
    let stderr_fd = config.stderr.map(|file| file.as_raw_fd());
    // we have to find mount paths (e.g.: input derivation output paths and their
    // runtime dependencies) before we enter the sandbox. That's because in the
    // sandbox we won't have access to pretty much anything.
    let mount_paths = get_mount_paths(config)?;
    // return value is the error code of the builder or 255 if anything went
    // wrong and we failed to execute the builder
    sandbox::run_in_sandbox(
        config.build_dir,
        || prepare_sandbox(config, &mount_paths),
        || run_build(config, stdout_fd, stderr_fd),
    )
}

pub fn build_derivation_command(derivation: &Derivation, build_dir: &Path) -> Command {
    // This function assumes that the sandbox is already fully set up
    let mut cmd = Command::new(&derivation.builder);
    cmd.args(&derivation.args)
        .envs(&derivation.env)
        .current_dir(build_dir);
    cmd
}

fn prepare_sandbox(config: &BuildConfig, mount_paths: &HashSet<PathBuf>) -> Result<(), String> {
    mount_standard_paths(config)?;
    mount_input_drvs(config, mount_paths)?;
    sandbox::mount_paths(
        config.derivation.input_srcs.iter().map(Path::new),
        config.build_dir,
    )
}

fn run_build(config: &BuildConfig, stdout_fd: Option<RawFd>, stderr_fd: Option<RawFd>) -> isize {
    let mut cmd = build_derivation_command(config.derivation, Path::new("/"));
    if let Some(stdout_fd) = stdout_fd {
        cmd.stdout(unsafe { Stdio::from_raw_fd(stdout_fd) });
    }
    if let Some(stderr_fd) = stderr_fd {
        cmd.stderr(unsafe { Stdio::from_raw_fd(stderr_fd) });
    }
    let exec_error = cmd.exec();
    // we should never get here because we exec into the builder above (i.e. the builder
    // process takes over). So, it's an error no matter what if we get here.
    eprintln!("Error executing builder: {}", exec_error);
    255
}

fn mount_input_drvs(config: &BuildConfig, mount_paths: &HashSet<PathBuf>) -> Result<(), String> {
    for path in mount_paths {
        sandbox::mount_path(path, config.build_dir)?;
    }
    Ok(())
}

fn get_mount_paths(config: &BuildConfig) -> Result<HashSet<PathBuf>, String> {
    let mut mount_paths = HashSet::new();
    for (drv_path, outputs) in &config.derivation.input_drvs {
        let derivation = load_derivation(drv_path)?;
        for output in outputs {
            let drv_output = derivation.outputs.get(output).ok_or_else(|| {
                format!("Could not find output '{output}' of derivation {drv_path}")
            })?;
            let drv_output_path = PathBuf::from(&drv_output.path);
            // We have to include direct runtime dependencies of input derivations. We don't need
            // to recurse transitively into input derivations of input derivations as these shouldn't
            // be needed.
            mount_paths.extend(config.deps_info.get_runtime_deps(&drv_output_path)?);
            mount_paths.insert(drv_output_path);
        }
    }
    Ok(mount_paths)
}

fn mount_standard_paths(config: &BuildConfig) -> Result<(), String> {
    sandbox::mount_path(Path::new("/dev/null"), config.build_dir)
}
