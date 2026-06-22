//! Move engine for the `move` binary.
//!
//! A move is "rename in place when possible, otherwise copy and delete the
//! source." `std::fs::rename` is attempted first — it is atomic and preserves
//! everything. When the rename cannot work atomically (the source and target
//! live on different filesystems, signalled by `EXDEV`) or when `--exclude`
//! means part of a directory must be left behind, the operation falls back to
//! the shared copy engine and then removes whatever was successfully copied.

use crate::cli::args::BackupMode;
use crate::cli::move_args::MoveOptions;
use crate::core::copy::copy;
use crate::error::{CopyError, CopyResult};
use crate::utility::backup::{create_backup, generate_backup_path};
use crate::utility::helper::prompt_overwrite;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

/// Disambiguates concurrent staging directories within a single process.
static STAGING_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Move a single `source` to `destination`, resolving "into directory" targets.
pub fn move_path(source: &Path, destination: &Path, options: &MoveOptions) -> CopyResult<()> {
    let source_metadata = std::fs::symlink_metadata(source)
        .map_err(|_| CopyError::InvalidSource(source.to_path_buf()))?;

    let target = resolve_target(source, destination);

    if target == source {
        return Err(CopyError::CopyFailed {
            source: source.to_path_buf(),
            destination: target,
            reason: "'source' and 'destination' are the same path".to_string(),
        });
    }

    // Overwrite policy is decided before any rename, since rename overwrites
    // silently. Precedence: no-clobber, then update, then interactive.
    if target_exists(&target) {
        if options.no_clobber {
            report_verbose(options, &format!("not overwriting '{}'", target.display()));
            return Ok(());
        }
        if options.update && !source_is_newer(source, &target)? {
            return Ok(());
        }
        if options.interactive && !prompt_overwrite(&target).map_err(CopyError::Io)? {
            return Ok(());
        }
        if let Some(mode) = options.backup
            && mode != BackupMode::None
        {
            let backup_path = generate_backup_path(&target, mode)?;
            create_backup(&target, &backup_path)?;
        }
    }

    let is_dir = source_metadata.is_dir();

    // Excludes can't be honored by an atomic rename of a whole directory, so a
    // directory move with excludes always takes the copy fallback.
    let force_fallback = is_dir && options.exclude_rules.is_some();

    if !force_fallback {
        match std::fs::rename(source, &target) {
            Ok(()) => {
                report_renamed(options, source, &target);
                return Ok(());
            }
            Err(error) if is_cross_device(&error) => {
                // Different filesystem: fall through to copy + delete.
            }
            Err(error) => {
                return Err(CopyError::CopyFailed {
                    source: source.to_path_buf(),
                    destination: target,
                    reason: error.to_string(),
                });
            }
        }
    }

    move_via_copy(source, &target, is_dir, options)
}

/// Move every `source` into the directory `destination`.
pub fn move_multiple(
    sources: Vec<PathBuf>,
    destination: PathBuf,
    options: &MoveOptions,
) -> CopyResult<()> {
    if !destination.is_dir() {
        return Err(CopyError::InvalidDestination(destination));
    }

    let mut failures = 0usize;
    for source in &sources {
        if options.abort.load(Ordering::Relaxed) {
            return Err(aborted());
        }
        if let Err(error) = move_path(source, &destination, options) {
            eprintln!("Failed to move '{}': {}", source.display(), error);
            failures += 1;
        }
    }

    if failures > 0 {
        return Err(CopyError::Io(io::Error::other(format!(
            "{failures} item(s) failed to move"
        ))));
    }
    Ok(())
}

/// Cross-device / exclude fallback: copy the source to the target, then remove
/// the source. The source is left untouched if the copy fails or is interrupted.
fn move_via_copy(
    source: &Path,
    target: &Path,
    is_dir: bool,
    options: &MoveOptions,
) -> CopyResult<()> {
    // The overwrite policy has already been applied, so clear any existing
    // target first to reproduce the source exactly at `target`.
    if target_exists(target) {
        remove_path(target)?;
    }

    if is_dir {
        // `copy` always nests a directory under its destination (dest/name),
        // so it can't rename. Stage the copy on the destination filesystem,
        // then cheap-rename the nested result into the final target.
        stage_and_place_dir(source, target, options)?;
    } else {
        // A file copies directly to the target path.
        copy(source, target, &options.to_copy_options())?;
    }

    // A user interrupt during the copy must not delete the source.
    if options.abort.load(Ordering::Relaxed) {
        return Err(aborted());
    }

    if is_dir {
        if options.exclude_rules.is_some() {
            // Excluded entries were never copied; delete only what reached the
            // target and prune directories that became empty, leaving the rest.
            remove_moved_sources(source, target)?;
        } else {
            std::fs::remove_dir_all(source).map_err(CopyError::Io)?;
        }
    } else {
        std::fs::remove_file(source).map_err(CopyError::Io)?;
    }

    report_renamed(options, source, target);
    Ok(())
}

/// Copy a directory across filesystems and place it at `target` under its final
/// name. The copy lands in a staging directory on the destination filesystem so
/// the concluding rename to `target` is cheap and atomic; staging is always
/// cleaned up, even on failure.
fn stage_and_place_dir(source: &Path, target: &Path, options: &MoveOptions) -> CopyResult<()> {
    let dest_parent = target.parent().unwrap_or_else(|| Path::new("."));
    let staging = unique_staging_dir(dest_parent);
    std::fs::create_dir(&staging).map_err(CopyError::Io)?;

    let result = place_via_staging(source, target, &staging, options);

    let _ = std::fs::remove_dir_all(&staging);
    result
}

fn place_via_staging(
    source: &Path,
    target: &Path,
    staging: &Path,
    options: &MoveOptions,
) -> CopyResult<()> {
    copy(source, staging, &options.to_copy_options())?;
    let name = source
        .file_name()
        .ok_or_else(|| CopyError::InvalidSource(source.to_path_buf()))?;
    std::fs::rename(staging.join(name), target).map_err(CopyError::Io)
}

fn unique_staging_dir(parent: &Path) -> PathBuf {
    let counter = STAGING_COUNTER.fetch_add(1, Ordering::Relaxed);
    parent.join(format!(
        ".copy-move-staging-{}-{}",
        std::process::id(),
        counter
    ))
}

/// Resolve the final target path, moving `source` *into* `destination` when the
/// destination is an existing directory.
fn resolve_target(source: &Path, destination: &Path) -> PathBuf {
    if destination.is_dir()
        && let Some(name) = source.file_name()
    {
        return destination.join(name);
    }
    destination.to_path_buf()
}

fn target_exists(path: &Path) -> bool {
    std::fs::symlink_metadata(path).is_ok()
}

fn source_is_newer(source: &Path, target: &Path) -> CopyResult<bool> {
    let source_time = std::fs::metadata(source)
        .and_then(|metadata| metadata.modified())
        .map_err(CopyError::Io)?;
    let target_time = std::fs::metadata(target)
        .and_then(|metadata| metadata.modified())
        .map_err(CopyError::Io)?;
    Ok(source_time > target_time)
}

fn is_cross_device(error: &io::Error) -> bool {
    error.raw_os_error() == Some(libc::EXDEV)
}

fn remove_path(path: &Path) -> CopyResult<()> {
    let metadata = std::fs::symlink_metadata(path).map_err(CopyError::Io)?;
    if metadata.is_dir() {
        std::fs::remove_dir_all(path).map_err(CopyError::Io)
    } else {
        std::fs::remove_file(path).map_err(CopyError::Io)
    }
}

/// After an exclude-aware directory copy, delete the source entries that reached
/// the target (proof they were moved) and prune directories left empty, while
/// preserving excluded files that were never copied.
fn remove_moved_sources(source: &Path, target: &Path) -> CopyResult<()> {
    remove_copied_entries(source, target).map_err(CopyError::Io)?;
    if is_empty_dir(source).map_err(CopyError::Io)? {
        std::fs::remove_dir(source).map_err(CopyError::Io)?;
    }
    Ok(())
}

fn remove_copied_entries(src: &Path, tgt: &Path) -> io::Result<()> {
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let src_child = entry.path();
        let tgt_child = tgt.join(entry.file_name());

        if file_type.is_dir() {
            remove_copied_entries(&src_child, &tgt_child)?;
            if is_empty_dir(&src_child)? {
                std::fs::remove_dir(&src_child)?;
            }
        } else if std::fs::symlink_metadata(&tgt_child).is_ok() {
            std::fs::remove_file(&src_child)?;
        }
    }
    Ok(())
}

fn is_empty_dir(path: &Path) -> io::Result<bool> {
    Ok(std::fs::read_dir(path)?.next().is_none())
}

fn aborted() -> CopyError {
    CopyError::Io(io::Error::new(
        io::ErrorKind::Interrupted,
        "Operation aborted by user",
    ))
}

fn report_renamed(options: &MoveOptions, source: &Path, target: &Path) {
    if options.verbose {
        println!("renamed '{}' -> '{}'", source.display(), target.display());
    }
}

fn report_verbose(options: &MoveOptions, message: &str) {
    if options.verbose {
        println!("{message}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utility::preserve::PreserveAttr;
    use std::fs;
    use std::sync::Arc;
    use std::sync::atomic::AtomicBool;
    use tempfile::TempDir;

    fn options() -> MoveOptions {
        MoveOptions {
            force: false,
            interactive: false,
            no_clobber: false,
            update: false,
            backup: None,
            verbose: false,
            parallel: 1,
            reflink: None,
            preserve: PreserveAttr::none(),
            exclude_rules: None,
            abort: Arc::new(AtomicBool::new(false)),
        }
    }

    #[test]
    fn moves_a_file_by_rename() {
        let dir = TempDir::new().unwrap();
        let source = dir.path().join("a.txt");
        let target = dir.path().join("b.txt");
        fs::write(&source, b"data").unwrap();

        move_path(&source, &target, &options()).unwrap();

        assert!(!source.exists());
        assert_eq!(fs::read(&target).unwrap(), b"data");
    }

    #[test]
    fn moves_into_an_existing_directory() {
        let dir = TempDir::new().unwrap();
        let source = dir.path().join("a.txt");
        let dest_dir = dir.path().join("dest");
        fs::write(&source, b"data").unwrap();
        fs::create_dir(&dest_dir).unwrap();

        move_path(&source, &dest_dir, &options()).unwrap();

        assert!(!source.exists());
        assert_eq!(fs::read(dest_dir.join("a.txt")).unwrap(), b"data");
    }

    #[test]
    fn renames_a_directory() {
        let dir = TempDir::new().unwrap();
        let source = dir.path().join("src");
        let target = dir.path().join("dst");
        fs::create_dir(&source).unwrap();
        fs::write(source.join("f.txt"), b"x").unwrap();

        move_path(&source, &target, &options()).unwrap();

        assert!(!source.exists());
        assert_eq!(fs::read(target.join("f.txt")).unwrap(), b"x");
    }

    #[test]
    fn no_clobber_keeps_existing_destination() {
        let dir = TempDir::new().unwrap();
        let source = dir.path().join("a.txt");
        let target = dir.path().join("b.txt");
        fs::write(&source, b"new").unwrap();
        fs::write(&target, b"old").unwrap();

        let mut opts = options();
        opts.no_clobber = true;
        move_path(&source, &target, &opts).unwrap();

        assert!(source.exists());
        assert_eq!(fs::read(&target).unwrap(), b"old");
    }

    #[test]
    fn copy_fallback_moves_a_file() {
        // Exercises the cross-device path directly (CI can't span filesystems).
        let dir = TempDir::new().unwrap();
        let source = dir.path().join("a.txt");
        let target = dir.path().join("b.txt");
        fs::write(&source, b"payload").unwrap();

        move_via_copy(&source, &target, false, &options()).unwrap();

        assert!(!source.exists());
        assert_eq!(fs::read(&target).unwrap(), b"payload");
    }

    #[test]
    fn copy_fallback_moves_a_directory() {
        let dir = TempDir::new().unwrap();
        let source = dir.path().join("src");
        let target = dir.path().join("dst");
        fs::create_dir_all(source.join("sub")).unwrap();
        fs::write(source.join("sub/f.txt"), b"deep").unwrap();

        move_via_copy(&source, &target, true, &options()).unwrap();

        assert!(!source.exists());
        assert_eq!(fs::read(target.join("sub/f.txt")).unwrap(), b"deep");
    }
}
