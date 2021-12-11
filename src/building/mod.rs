use crate::derivations::Derivation;
use libc;
use nix::sys::wait;
use nix::{mount, sched, unistd};
use std::fs;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn build_derivation_command(derivation: &Derivation, build_dir: &Path) -> Command {
    // This function assumes that the sandbox is already fully set up
    let mut cmd = Command::new(&derivation.builder);
    cmd.args(&derivation.args)
        .envs(&derivation.env)
        .current_dir(build_dir);
    cmd
}

pub fn build_derivation_sandboxed(
    derivation: &Derivation,
    build_dir: &Path,
) -> Result<i32, String> {
    // this function assumes all derivation inputs are present and won't be
    // GC'd for the duration of this build

    // return value is the error code of the builder or 255 if anything went
    // wrong and we failed to execute the builder

    // Stdout: completely silent
    // TODO: Stderr: logs of this command (and optionally stdout and/or stderr of the build)
    // --builder-stderr FD: the file descriptor to which redirect builder's stderr (otherwise redirected to stderr)
    // --builder-stdout FD: the file descriptor to which redirect builder's stdout (otherwise redirected to stderr)

    // the builder's working directory will be in `{build_dir}/`

    let builder_func = || -> isize {
        let sandbox_result = enter_sandbox(derivation, build_dir);
        if let Err(sandbox_error) = sandbox_result {
            eprintln!("Error setting up builder sandbox: {}", sandbox_error);
            return 255;
        }
        let exec_error = build_derivation_command(&derivation, &Path::new("/")).exec();
        // we should never get here because we exec into the builder above (i.e. the builder
        // process takes over). So, it's an error no matter what if we get here.
        eprintln!("Error executing builder: {:?}", exec_error);
        return 255;
    };

    let pid = sched::clone(
        Box::new(builder_func),
        &mut vec![0u8; 1024 * 1024],
        sched::CloneFlags::CLONE_NEWNS | sched::CloneFlags::CLONE_NEWUSER,
        Some(libc::SIGCHLD),
    )
    .map_err(|err| format!("Failed to start the builder process. Error: {:?}", err))?;

    match wait::waitpid(pid, None) {
        Ok(wait::WaitStatus::Exited(_, exit_code)) => Ok(exit_code),
        Ok(wait::WaitStatus::Signaled(_, signal, core_dumped)) => {
            eprintln!(
                "Builder killed by signal {} (core dumped: {})",
                signal, core_dumped,
            );
            Ok(255)
        }
        Ok(state) => {
            eprintln!("Unexpected builder process state: {:?}", state);
            Ok(255)
        }
        Err(err) => {
            eprintln!("Error waiting for the builder process: {:?}", err);
            Ok(255)
        }
    }
}

fn enter_sandbox(derivation: &Derivation, build_dir: &Path) -> Result<(), String> {
    mount_paths(derivation.input_srcs.iter().map(Path::new), build_dir)?;
    pivot_root(build_dir)
}

fn mount_paths<'a>(paths: impl Iterator<Item = &'a Path>, target_dir: &Path) -> Result<(), String> {
    for path in paths {
        let target_path = prepare_mount_path(path, target_dir)?;
        mount::mount(
            Some(path),
            &target_path,
            None::<&str>,
            mount::MsFlags::MS_BIND | mount::MsFlags::MS_REC,
            None::<&str>,
        )
        .map_err(|e| {
            format!(
                "Failed to bind mount {:?} to {:?}. Error: {:?}",
                path, target_path, e
            )
        })?;
    }
    Ok(())
}

fn pivot_root(new_root: &Path) -> Result<(), String> {
    mount_rootfs(new_root)?;
    let old_root = new_root.join("oldroot");
    fs::create_dir_all(&old_root).map_err(|e| format!("Error creating oldroot: {:?}", e))?;
    unistd::pivot_root(new_root, &old_root)
        .map_err(|e| format!("Error pivoting to new root: {:?}", e))?;
    unistd::chdir("/").map_err(|e| format!("Error cd'ing to new root: {:?}", e))?;
    mount::umount2("/oldroot", mount::MntFlags::MNT_DETACH)
        .map_err(|e| format!("Error unmounting old root: {:?}", e))?;
    std::fs::remove_dir_all("/oldroot").map_err(|e| format!("Error removing old root: {:?}", e))
}

fn mount_rootfs(rootfs: &Path) -> Result<(), String> {
    // we have to mount the old root as part of requirements of the `pivot_root` syscall.
    // For more info see: https://man7.org/linux/man-pages/man2/pivot_root.2.html
    mount::mount(
        Some("/"),
        "/",
        None::<&str>,
        mount::MsFlags::MS_PRIVATE | mount::MsFlags::MS_REC,
        None::<&str>,
    )
    .map_err(|e| format!("Error mounting old root: {:?}", e))?;

    mount::mount(
        Some(rootfs),
        rootfs,
        None::<&str>,
        mount::MsFlags::MS_BIND | mount::MsFlags::MS_REC,
        None::<&str>,
    )
    .map_err(|e| format!("Failed to mount new root: {:?}", e))
}

fn prepare_mount_path(source_path: &Path, target_root_dir: &Path) -> Result<PathBuf, String> {
    let path_without_root = source_path.strip_prefix("/").map_err(|e| {
        format!(
            "Could not remove '/' from path {:?}. Error: {:?}",
            source_path, e
        )
    })?;
    let target_path = target_root_dir.join(path_without_root);
    if source_path.is_dir() {
        fs::create_dir_all(&target_path)
            .map_err(|e| format!("Error creating directory {:?}: {:?}", target_path, e))?;
    } else {
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                format!(
                    "Error creating parent directories for {:?}: {:?}",
                    source_path, e
                )
            })?;
        }
        fs::write(&target_path, "").map_err(|e| {
            format!(
                "Error creating empty target file {:?}: {:?}",
                source_path, e
            )
        })?;
    }
    return Ok(target_path);
}
