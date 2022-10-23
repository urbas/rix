use crate::derivations::{load_derivation, Derivation};
use crate::sandbox::{mount_path, mount_paths, run_in_sandbox};
use std::env::set_var;
use std::fs::File;
use std::os::unix::io::{AsRawFd, FromRawFd, RawFd};
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Command, Stdio};

pub struct BuildConfig<'a> {
    derivation: &'a Derivation,
    build_dir: &'a Path,
    stdout: Option<&'a File>,
    stderr: Option<&'a File>,
}

impl<'a> BuildConfig<'a> {
    pub fn new(derivation: &'a Derivation, build_dir: &'a Path) -> BuildConfig {
        BuildConfig {
            derivation: derivation,
            build_dir: build_dir,
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
    // return value is the error code of the builder or 255 if anything went
    // wrong and we failed to execute the builder
    run_in_sandbox(
        config.build_dir,
        || prepare_sandbox(config),
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

fn prepare_sandbox(config: &BuildConfig) -> Result<(), String> {
    set_env(&config.derivation);
    mount_input_drvs(config)?;
    mount_paths(
        config.derivation.input_srcs.iter().map(Path::new),
        config.build_dir,
    )
}

fn run_build(config: &BuildConfig, stdout_fd: Option<RawFd>, stderr_fd: Option<RawFd>) -> isize {
    let mut cmd = build_derivation_command(&config.derivation, &Path::new("/"));
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

fn set_env(derivation: &Derivation) {
    for (var_name, var_value) in &derivation.env {
        set_var(var_name, var_value);
    }
}

fn mount_input_drvs(config: &BuildConfig) -> Result<(), String> {
    for (drv_path, outputs) in &config.derivation.input_drvs {
        let derivation = load_derivation(drv_path)?;
        for output in outputs {
            let drv_output = derivation.outputs.get(output).ok_or_else(|| {
                format!(
                    "Could not find output '{}' of derivation {:?}",
                    output, drv_path
                )
            })?;
            mount_path(Path::new(&drv_output.path), config.build_dir)?;
        }
    }
    Ok(())
}
