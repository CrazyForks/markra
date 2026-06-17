use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const COMMAND_NAME: &str = "markra";
const MANAGED_MARKER: &str = "Managed by Markra";
const TARGET_PREFIX: &str = "target: ";

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ShellCommandStatus {
    command_path: Option<String>,
    target_path: Option<String>,
    status: String,
}

fn shell_command_file_name() -> &'static str {
    if cfg!(windows) {
        "markra.cmd"
    } else {
        COMMAND_NAME
    }
}

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

fn executable_target_path() -> Result<PathBuf, String> {
    if let Some(appimage) = env::var_os("APPIMAGE") {
        let path = PathBuf::from(appimage);
        if path.is_file() {
            return Ok(path);
        }
    }

    env::current_exe().map_err(|error| error.to_string())
}

fn path_entries() -> Vec<PathBuf> {
    env::var_os("PATH")
        .map(|path| env::split_paths(&path).collect())
        .unwrap_or_default()
}

fn command_candidates_for_dir(dir: &Path) -> Vec<PathBuf> {
    if cfg!(windows) {
        ["markra.cmd", "markra.exe", "markra.bat", "markra"]
            .into_iter()
            .map(|file_name| dir.join(file_name))
            .collect()
    } else {
        vec![dir.join(COMMAND_NAME)]
    }
}

fn existing_command_in_path() -> Option<PathBuf> {
    path_entries()
        .into_iter()
        .flat_map(|dir| command_candidates_for_dir(&dir))
        .find(|path| path.is_file())
}

fn directory_is_writable(path: &Path) -> bool {
    path.is_dir()
        && fs::metadata(path)
            .map(|metadata| !metadata.permissions().readonly())
            .unwrap_or(false)
}

fn directory_is_installable(path: &Path) -> bool {
    if path.is_dir() {
        return directory_is_writable(path);
    }

    path.ancestors()
        .skip(1)
        .find(|ancestor| ancestor.exists())
        .is_some_and(directory_is_writable)
}

fn standard_install_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    if cfg!(target_os = "macos") {
        dirs.push(PathBuf::from("/opt/homebrew/bin"));
        dirs.push(PathBuf::from("/usr/local/bin"));
    }

    if let Some(home) = dirs::home_dir() {
        dirs.push(home.join(".local/bin"));
        dirs.push(home.join("bin"));
    }

    if cfg!(target_os = "linux") {
        dirs.push(PathBuf::from("/usr/local/bin"));
    }

    #[cfg(windows)]
    if let Some(local_app_data) = env::var_os("LOCALAPPDATA") {
        dirs.push(PathBuf::from(local_app_data).join("Markra").join("bin"));
    }

    dirs
}

fn preferred_install_dir() -> Option<PathBuf> {
    if let Some(dir) = env::var_os("MARKRA_SHELL_COMMAND_DIR").map(PathBuf::from) {
        return Some(dir);
    }

    standard_install_dirs()
        .into_iter()
        .chain(path_entries())
        .find(|path| directory_is_installable(path))
}

fn preferred_command_path() -> Option<PathBuf> {
    preferred_install_dir().map(|dir| dir.join(shell_command_file_name()))
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn command_script(target_path: &Path) -> String {
    let target = path_to_string(target_path);

    if cfg!(windows) {
        return format!(
            "@echo off\r\nREM {MANAGED_MARKER}\r\nREM {TARGET_PREFIX}{target}\r\n\"{target}\" %*\r\n"
        );
    }

    format!(
        "#!/bin/sh\n# {MANAGED_MARKER}\n# {TARGET_PREFIX}{target}\nexec {} \"$@\"\n",
        shell_quote(&target)
    )
}

fn managed_target_from_command(path: &Path) -> Option<PathBuf> {
    let content = fs::read_to_string(path).ok()?;
    if !content.contains(MANAGED_MARKER) {
        return None;
    }

    content.lines().find_map(|line| {
        let normalized = line
            .trim()
            .trim_start_matches('#')
            .trim_start_matches("REM")
            .trim();
        normalized
            .strip_prefix(TARGET_PREFIX)
            .map(|target| PathBuf::from(target.trim()))
    })
}

fn paths_match(left: &Path, right: &Path) -> bool {
    match (left.canonicalize(), right.canonicalize()) {
        (Ok(left), Ok(right)) => left == right,
        _ => left == right,
    }
}

fn status_value(
    command_path: Option<&Path>,
    target_path: Option<&Path>,
    status: &str,
) -> ShellCommandStatus {
    ShellCommandStatus {
        command_path: command_path.map(path_to_string),
        target_path: target_path.map(path_to_string),
        status: status.to_string(),
    }
}

fn shell_command_status_for_target(target_path: &Path) -> ShellCommandStatus {
    if let Some(command_path) = existing_command_in_path() {
        if let Some(managed_target) = managed_target_from_command(&command_path) {
            let status = if paths_match(&managed_target, target_path) {
                "installed"
            } else {
                "needsRepair"
            };
            return status_value(Some(&command_path), Some(target_path), status);
        }

        return status_value(Some(&command_path), Some(target_path), "conflict");
    }

    let Some(command_path) = preferred_command_path() else {
        return status_value(None, Some(target_path), "unavailable");
    };

    if command_path.is_file() {
        if let Some(managed_target) = managed_target_from_command(&command_path) {
            let status = if paths_match(&managed_target, target_path) {
                "installed"
            } else {
                "needsRepair"
            };
            return status_value(Some(&command_path), Some(target_path), status);
        }

        return status_value(Some(&command_path), Some(target_path), "conflict");
    }

    status_value(Some(&command_path), Some(target_path), "missing")
}

fn shell_command_status() -> ShellCommandStatus {
    match executable_target_path() {
        Ok(target_path) => shell_command_status_for_target(&target_path),
        Err(_) => status_value(None, None, "unavailable"),
    }
}

fn install_command_at(command_path: &Path, target_path: &Path) -> Result<(), String> {
    if command_path.is_file() && managed_target_from_command(command_path).is_none() {
        return Err(format!(
            "A different markra command already exists at {}.",
            path_to_string(command_path)
        ));
    }

    let parent = command_path
        .parent()
        .ok_or_else(|| "Command path is invalid.".to_string())?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    fs::write(command_path, command_script(target_path)).map_err(|error| error.to_string())?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let mut permissions = fs::metadata(command_path)
            .map_err(|error| error.to_string())?
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(command_path, permissions).map_err(|error| error.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub(crate) fn get_shell_command_status() -> ShellCommandStatus {
    shell_command_status()
}

#[tauri::command]
pub(crate) fn install_shell_command() -> Result<ShellCommandStatus, String> {
    let target_path = executable_target_path()?;
    let status = shell_command_status_for_target(&target_path);

    if status.status == "conflict" {
        return Err("A different markra command already exists on PATH.".to_string());
    }

    let command_path = status
        .command_path
        .as_deref()
        .map(PathBuf::from)
        .or_else(preferred_command_path)
        .ok_or_else(|| {
            "No writable PATH directory is available for installing markra.".to_string()
        })?;

    install_command_at(&command_path, &target_path)?;
    Ok(shell_command_status_for_target(&target_path))
}

#[tauri::command]
pub(crate) fn uninstall_shell_command() -> Result<ShellCommandStatus, String> {
    let target_path = executable_target_path()?;
    let status = shell_command_status_for_target(&target_path);

    let Some(command_path) = status.command_path.as_deref().map(PathBuf::from) else {
        return Ok(status);
    };

    if command_path.is_file() {
        if managed_target_from_command(&command_path).is_none() {
            return Err("The markra command on PATH is not managed by Markra.".to_string());
        }

        fs::remove_file(&command_path).map_err(|error| error.to_string())?;
    }

    Ok(shell_command_status_for_target(&target_path))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_root(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "markra-shell-command-{name}-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after epoch")
                .as_nanos()
        ));
        fs::create_dir_all(&root).expect("test root should be created");
        root
    }

    #[test]
    fn creates_a_managed_shell_script_for_the_markra_executable() {
        let target = PathBuf::from("/mock-app/Markra.app/Contents/MacOS/markra");
        let script = command_script(&target);

        assert!(script.contains(MANAGED_MARKER));
        assert!(script.contains("target: /mock-app/Markra.app/Contents/MacOS/markra"));
        assert!(script.contains("exec '/mock-app/Markra.app/Contents/MacOS/markra' \"$@\""));
    }

    #[test]
    fn detects_repair_when_a_managed_command_points_to_an_old_target() {
        let root = test_root("repair");
        let command_path = root.join(shell_command_file_name());
        let old_target = root.join("old-markra");
        let next_target = root.join("next-markra");
        fs::write(&old_target, "").expect("old target should be created");
        fs::write(&next_target, "").expect("next target should be created");
        install_command_at(&command_path, &old_target).expect("command should install");

        let status = if let Some(managed_target) = managed_target_from_command(&command_path) {
            let status = if paths_match(&managed_target, &next_target) {
                "installed"
            } else {
                "needsRepair"
            };
            status_value(Some(&command_path), Some(&next_target), status)
        } else {
            status_value(Some(&command_path), Some(&next_target), "conflict")
        };

        assert_eq!(status.status, "needsRepair");

        fs::remove_dir_all(root).expect("test root should be removed");
    }
}
