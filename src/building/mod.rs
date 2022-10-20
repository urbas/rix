use crate::derivations::Derivation;
use crate::sandbox::{mount_paths, run_in_sandbox};
use std::fs::File;
use std::os::unix::io::{AsRawFd, FromRawFd};
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
        |new_root| mount_paths(config.derivation.input_srcs.iter().map(Path::new), new_root),
        || {
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
        },
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
