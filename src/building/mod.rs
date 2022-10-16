use crate::derivations::Derivation;
use crate::sandbox::{mount_paths, run_in_sandbox};
use std::fs::File;
use std::os::unix::io::FromRawFd;
use std::os::unix::io::{AsRawFd, RawFd};
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Command, Stdio};

pub struct BuildConfig<'a> {
    derivation: &'a Derivation,
    build_dir: &'a Path,
    stdout_fd: Option<RawFd>,
    stderr_fd: Option<RawFd>,
}

impl<'a> BuildConfig<'a> {
    pub fn new(derivation: &'a Derivation, build_dir: &'a Path) -> BuildConfig {
        BuildConfig {
            derivation: derivation,
            build_dir: build_dir,
            stderr_fd: None,
            stdout_fd: None,
        }
    }

    pub fn stdout_to_file(&mut self, file: &'a File) -> &mut Self {
        self.stdout_fd = Some(file.as_raw_fd());
        self
    }

    pub fn stderr_to_file(&mut self, file: &'a File) -> &mut Self {
        self.stderr_fd = Some(file.as_raw_fd());
        self
    }
}

pub fn build_derivation_sandboxed(config: &BuildConfig) -> Result<i32, String> {
    // this function assumes all derivation inputs are present and won't be
    // GC'd for the duration of this build

    // return value is the error code of the builder or 255 if anything went
    // wrong and we failed to execute the builder
    run_in_sandbox(
        config.build_dir,
        |new_root| mount_paths(config.derivation.input_srcs.iter().map(Path::new), new_root),
        || {
            let mut cmd = build_derivation_command(&config.derivation, &Path::new("/"));
            if let Some(stdout_fd) = config.stdout_fd {
                cmd.stdout(unsafe { Stdio::from_raw_fd(stdout_fd) });
            }
            if let Some(stderr_fd) = config.stderr_fd {
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
