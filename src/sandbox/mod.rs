use nix::env::clearenv;
use nix::sys::wait;
use nix::{mount, sched, unistd};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

pub fn run_in_sandbox(
    new_root: &Path,
    prepare_sandbox: impl Fn() -> Result<(), String>,
    run: impl Fn() -> isize,
) -> Result<i32, String> {
    let forked_logic = || -> isize {
        if let Err(err) = unsafe { clearenv() } {
            eprintln!("Could not clear environment variables: {err}");
            return 255;
        }
        if let Err(err) = prepare_sandbox() {
            eprintln!("Error preparing the sandbox: {err}");
            return 255;
        }
        if let Err(err) = pivot_root(new_root) {
            eprintln!("Error setting up builder sandbox: {err}");
            return 255;
        }
        run()
    };

    let pid = sched::clone(
        Box::new(forked_logic),
        &mut vec![0u8; 1024 * 1024],
        sched::CloneFlags::CLONE_NEWNS | sched::CloneFlags::CLONE_NEWUSER,
        Some(libc::SIGCHLD),
    )
    .map_err(|err| format!("Failed to start the builder process. Error: {err}"))?;

    match wait::waitpid(pid, None) {
        Ok(wait::WaitStatus::Exited(_, exit_code)) => Ok(exit_code),
        Ok(wait::WaitStatus::Signaled(_, signal, core_dumped)) => {
            eprintln!("Builder killed by signal {signal} (core dumped: {core_dumped})");
            Ok(255)
        }
        Ok(state) => {
            eprintln!("Unexpected builder process state: {state:?}");
            Ok(255)
        }
        Err(err) => {
            eprintln!("Error waiting for the builder process: {err}");
            Ok(255)
        }
    }
}

pub fn mount_paths<'a>(
    paths: impl Iterator<Item = &'a Path>,
    new_root: &Path,
) -> Result<(), String> {
    for path in paths {
        mount_path(path, new_root)?;
    }
    Ok(())
}

pub fn mount_path(path: &Path, new_root: &Path) -> Result<(), String> {
    let target_path = prepare_mount_path(path, new_root)?;
    mount::mount(
        Some(path),
        &target_path,
        None::<&str>,
        mount::MsFlags::MS_BIND | mount::MsFlags::MS_REC,
        None::<&str>,
    )
    .map_err(|e| format!("Failed to bind mount {path:?} to {target_path:?}. Error: {e}"))
}

pub fn pivot_root(new_root: &Path) -> Result<(), String> {
    mount_rootfs(new_root)?;
    let old_root_name = Uuid::new_v4().to_string();
    let old_root = new_root.join(&old_root_name);
    let old_root_absolute = Path::new("/").join(old_root_name);
    fs::create_dir_all(&old_root).map_err(|e| format!("Error creating oldroot: {e}"))?;
    unistd::chdir(new_root).map_err(|e| format!("Error cd'ing to new root: {e}"))?;
    unistd::pivot_root(".", &old_root).map_err(|e| format!("Error pivoting to new root: {e}"))?;
    // It looks like we have to call `chroot` after `pivot_root`: https://superuser.com/questions/1575316/usage-of-chroot-after-pivot-root
    unistd::chroot(".").map_err(|e| format!("Failed to chroot. {e}"))?;
    mount::umount2(&old_root_absolute, mount::MntFlags::MNT_DETACH)
        .map_err(|e| format!("Error unmounting old root: {e}"))?;
    std::fs::remove_dir_all(&old_root_absolute).map_err(|e| format!("Error removing old root: {e}"))
}

fn mount_rootfs(new_root: &Path) -> Result<(), String> {
    // we have to mount the old root as part of requirements of the `pivot_root` syscall.
    // For more info see: https://man7.org/linux/man-pages/man2/pivot_root.2.html
    mount::mount(
        Some("/"),
        "/",
        None::<&str>,
        mount::MsFlags::MS_PRIVATE | mount::MsFlags::MS_REC,
        None::<&str>,
    )
    .map_err(|e| format!("Error mounting old root: {e}"))?;

    mount::mount(
        Some(new_root),
        new_root,
        None::<&str>,
        mount::MsFlags::MS_BIND | mount::MsFlags::MS_REC,
        None::<&str>,
    )
    .map_err(|e| format!("Failed to mount new root: {e}"))
}

fn prepare_mount_path(source_path: &Path, new_root: &Path) -> Result<PathBuf, String> {
    let path_without_root = source_path
        .strip_prefix("/")
        .map_err(|e| format!("Could not remove '/' from path {source_path:?}. Error: {e}"))?;
    let target_path = new_root.join(path_without_root);
    if source_path.is_dir() {
        fs::create_dir_all(&target_path)
            .map_err(|e| format!("Error creating directory {:?}: {}", target_path, e))?;
    } else {
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                format!("Error creating parent directories for {source_path:?}: {e}")
            })?;
        }
        fs::write(&target_path, "")
            .map_err(|e| format!("Error creating empty target file {source_path:?}: {e}"))?;
    }
    Ok(target_path)
}
