//! The file replacement shared by macOS bundles and Linux AppImages.

use std::fs;
use std::io;
use std::path::Path;

pub(super) fn replace_and_relaunch(
    current: &Path,
    replacement: &Path,
    stage: &Path,
    backup_name: &str,
    launch: impl Fn(&Path) -> Result<(), String>,
    report: impl Fn(&Result<(), String>),
) -> Result<(), String> {
    replace_with(
        current,
        replacement,
        stage,
        backup_name,
        launch,
        report,
        |from, to| fs::rename(from, to),
    )
}

fn replace_with(
    current: &Path,
    replacement: &Path,
    stage: &Path,
    backup_name: &str,
    launch: impl Fn(&Path) -> Result<(), String>,
    report: impl Fn(&Result<(), String>),
    rename: impl Fn(&Path, &Path) -> io::Result<()>,
) -> Result<(), String> {
    let backup = stage.join(backup_name);
    if backup.exists() {
        let result = Err(format!(
            "the update staging backup already exists: {}",
            backup.display()
        ));
        report(&result);
        return result;
    }
    if let Err(error) = rename(current, &backup) {
        let _ = fs::remove_dir_all(stage);
        return report_and_relaunch(
            current,
            format!("moving the current app aside: {error}"),
            &launch,
            &report,
        );
    }
    let failure = match rename(replacement, current) {
        Err(error) => format!("putting the staged app in place: {error}"),
        Ok(()) => match launch(current) {
            Ok(()) => {
                let _ = remove_path(&backup);
                // A failed backup removal must leave the recovery copy intact.
                let _ = super::update_stage::remove_stage(stage);
                report(&Ok(()));
                return Ok(());
            }
            Err(error) => {
                if let Err(remove) = remove_path(current) {
                    let result = Err(format!(
                        "{error}; removing the failed replacement: {remove}; backup preserved at {}",
                        backup.display()
                    ));
                    report(&result);
                    return result;
                }
                error
            }
        },
    };
    match rename(&backup, current) {
        Ok(()) => {
            let _ = fs::remove_dir_all(stage);
            report_and_relaunch(current, failure, &launch, &report)
        }
        Err(restore) => {
            let result = Err(format!(
                "{failure}; restoring the previous app: {restore}; backup preserved at {}",
                backup.display()
            ));
            report(&result);
            result
        }
    }
}

pub(super) fn report_and_relaunch(
    current: &Path,
    error: String,
    launch: &impl Fn(&Path) -> Result<(), String>,
    report: &impl Fn(&Result<(), String>),
) -> Result<(), String> {
    let mut result = Err(error);
    // The old GUI reads its failure at startup, before the launch health check
    // can finish. Record it first; append a failed relaunch if necessary.
    report(&result);
    if let Err(relaunch) = launch(current) {
        result = Err(format!(
            "{}; relaunching the previous app: {relaunch}",
            result.unwrap_err()
        ));
        report(&result);
    }
    result
}

fn remove_path(path: &Path) -> io::Result<()> {
    if path.is_dir() {
        fs::remove_dir_all(path)
    } else {
        fs::remove_file(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[test]
    fn a_successful_replacement_cleans_the_stage() {
        let root = tempfile::tempdir().unwrap();
        let current = root.path().join("app");
        let stage = root.path().join("stage");
        fs::create_dir(&stage).unwrap();
        let replacement = stage.join("new");
        fs::write(&current, b"old").unwrap();
        fs::write(&replacement, b"new").unwrap();
        replace_and_relaunch(
            &current,
            &replacement,
            &stage,
            "previous.app",
            |_| Ok(()),
            |_| (),
        )
        .unwrap();
        assert_eq!(fs::read(current).unwrap(), b"new");
        assert!(!stage.exists());
    }

    #[test]
    fn failed_placement_preserves_the_backup_when_restore_fails() {
        let root = tempfile::tempdir().unwrap();
        let current = root.path().join("app");
        let stage = root.path().join("stage");
        fs::create_dir(&stage).unwrap();
        let replacement = stage.join("new");
        fs::write(&current, b"previous version").unwrap();
        fs::write(&replacement, b"new version").unwrap();
        let backup = stage.join("previous.AppImage");
        let error = replace_with(
            &current,
            &replacement,
            &stage,
            "previous.AppImage",
            |_| panic!("there is no restored app to launch"),
            |_| (),
            |from, to| {
                if from == replacement || from == backup {
                    return Err(io::Error::new(
                        io::ErrorKind::PermissionDenied,
                        "injected rename failure",
                    ));
                }
                fs::rename(from, to)
            },
        )
        .unwrap_err();
        assert!(
            backup.exists(),
            "recovery deleted the only previous version: {error}"
        );
        assert_eq!(fs::read(&backup).unwrap(), b"previous version");
        assert!(error.contains(&backup.display().to_string()), "{error}");
    }

    #[test]
    fn failed_placement_reports_then_relaunches_the_restored_app() {
        let root = tempfile::tempdir().unwrap();
        let current = root.path().join("app");
        let stage = root.path().join("stage");
        fs::create_dir(&stage).unwrap();
        let replacement = stage.join("new");
        fs::write(&current, b"previous version").unwrap();
        let reported = Cell::new(false);
        let launched = Cell::new(false);
        let result = replace_with(
            &current,
            &replacement,
            &stage,
            "previous.app",
            |path| {
                assert!(
                    reported.get(),
                    "the relaunched app must find the failure on disk"
                );
                assert_eq!(fs::read(path).unwrap(), b"previous version");
                launched.set(true);
                Ok(())
            },
            |_| reported.set(true),
            |from, to| fs::rename(from, to),
        );
        assert!(result.is_err());
        assert!(launched.get(), "the old app stayed closed after recovery");
        assert!(
            !stage.exists(),
            "a successful recovery should clean its stage"
        );
    }
}
