use std::{ffi::OsStr, fs, io, path::Path};

const UPDATER_PREFIX: &str = "CLI Hub-";
const UPDATER_MARKER: &str = "-updater-";

pub fn clean_stale_artifacts() {
    let _ = clean_directory(&std::env::temp_dir());
}

fn clean_directory(directory: &Path) -> io::Result<usize> {
    let mut removed = 0;

    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        if !is_cli_hub_updater_artifact(&entry.file_name()) {
            continue;
        }

        let path = entry.path();
        let result = if path.is_dir() {
            fs::remove_dir_all(&path)
        } else {
            fs::remove_file(&path)
        };

        if result.is_ok() {
            removed += 1;
        }
    }

    Ok(removed)
}

fn is_cli_hub_updater_artifact(name: &OsStr) -> bool {
    name.to_str()
        .is_some_and(|name| name.starts_with(UPDATER_PREFIX) && name.contains(UPDATER_MARKER))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn cleanup_only_removes_cli_hub_updater_artifacts() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after the Unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("cli-hub-cleanup-test-{unique}"));
        fs::create_dir_all(&root).expect("test directory should be created");

        let updater_dir = root.join("CLI Hub-0.1.2-updater-example");
        let unrelated_dir = root.join("another-app-0.1.2-updater-example");
        let user_file = root.join("CLI Hub installer.exe");
        fs::create_dir_all(&updater_dir).expect("updater directory should be created");
        fs::create_dir_all(&unrelated_dir).expect("unrelated directory should be created");
        fs::write(&user_file, b"keep").expect("unrelated file should be created");

        assert_eq!(clean_directory(&root).expect("cleanup should succeed"), 1);
        assert!(!updater_dir.exists());
        assert!(unrelated_dir.exists());
        assert!(user_file.exists());

        fs::remove_dir_all(root).expect("test directory should be removed");
    }
}
