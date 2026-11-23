//! Turns a rice into a list of files and writes them, backing up what was there before.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::generators::{fastfetch, komorebi, oh_my_posh, terminal, yasb};
use crate::rice::{Rice, Terminal};
use crate::text::EditError;

/// Where each tool keeps its configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Paths {
    pub komorebi: PathBuf,
    pub yasb_styles: PathBuf,
    pub oh_my_posh_theme: PathBuf,
    pub fastfetch: PathBuf,
    pub windows_terminal: PathBuf,
    pub wezterm_module: PathBuf,
    pub alacritty_import: PathBuf,
}

impl Paths {
    /// Default locations on Windows, relative to `%USERPROFILE%` and `%LOCALAPPDATA%`.
    /// komorebi honours `KOMOREBI_CONFIG_HOME` when it is set.
    pub fn windows(user_profile: &Path, local_app_data: &Path, komorebi_home: Option<&Path>) -> Self {
        let config = user_profile.join(".config");
        Self {
            komorebi: komorebi_home.unwrap_or(user_profile).join("komorebi.json"),
            yasb_styles: config.join("yasb").join("styles.css"),
            oh_my_posh_theme: config.join("kazari").join("kazari.omp.json"),
            fastfetch: config.join("fastfetch").join("config.jsonc"),
            windows_terminal: local_app_data
                .join("Packages")
                .join("Microsoft.WindowsTerminal_8wekyb3d8bbwe")
                .join("LocalState")
                .join("settings.json"),
            wezterm_module: config.join("wezterm").join("kazari.lua"),
            alacritty_import: user_profile
                .join("AppData")
                .join("Roaming")
                .join("alacritty")
                .join("kazari.toml"),
        }
    }

    /// Reads the Windows locations from the environment. `None` outside Windows.
    pub fn from_env() -> Option<Self> {
        if !cfg!(windows) {
            return None;
        }
        let user_profile = PathBuf::from(std::env::var_os("USERPROFILE")?);
        let local_app_data = PathBuf::from(std::env::var_os("LOCALAPPDATA")?);
        let komorebi_home = std::env::var_os("KOMOREBI_CONFIG_HOME").map(PathBuf::from);
        Some(Self::windows(
            &user_profile,
            &local_app_data,
            komorebi_home.as_deref(),
        ))
    }

    /// The same file names under one folder, to preview a rice without touching the system.
    pub fn export(dir: &Path) -> Self {
        Self {
            komorebi: dir.join("komorebi.json"),
            yasb_styles: dir.join("yasb").join("styles.css"),
            oh_my_posh_theme: dir.join("kazari.omp.json"),
            fastfetch: dir.join("fastfetch").join("config.jsonc"),
            windows_terminal: dir.join("windows-terminal").join("settings.json"),
            wezterm_module: dir.join("wezterm").join("kazari.lua"),
            alacritty_import: dir.join("alacritty").join("kazari.toml"),
        }
    }
}

/// One file kazari wants to write.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedFile {
    pub path: PathBuf,
    pub contents: String,
}

#[derive(Debug, thiserror::Error)]
pub enum PlanError {
    #[error("{path}: {source}")]
    Edit { path: PathBuf, source: EditError },
    #[error("{path}: {source}")]
    Io { path: PathBuf, source: io::Error },
}

/// Builds every file for a rice. `read` returns the current contents of a file (empty when it
/// does not exist yet), so existing configs are patched instead of replaced.
pub fn plan(
    rice: &Rice,
    paths: &Paths,
    read: impl Fn(&Path) -> String,
) -> Result<Vec<PlannedFile>, PlanError> {
    let edit_err = |path: &Path| {
        let path = path.to_path_buf();
        move |source| PlanError::Edit { path, source }
    };
    let file = |path: &Path, contents: String| PlannedFile {
        path: path.to_path_buf(),
        contents,
    };

    let mut files = vec![
        file(
            &paths.komorebi,
            komorebi::patch(&read(&paths.komorebi), rice).map_err(edit_err(&paths.komorebi))?,
        ),
        file(&paths.yasb_styles, yasb::patch(&read(&paths.yasb_styles), rice)),
        file(&paths.oh_my_posh_theme, oh_my_posh::theme(rice)),
        file(&paths.fastfetch, fastfetch::config(rice)),
    ];
    files.push(match rice.terminal {
        Terminal::WindowsTerminal => file(
            &paths.windows_terminal,
            terminal::patch_windows_terminal(&read(&paths.windows_terminal), rice)
                .map_err(edit_err(&paths.windows_terminal))?,
        ),
        Terminal::WezTerm => file(&paths.wezterm_module, terminal::wezterm_module(rice)),
        Terminal::Alacritty => file(&paths.alacritty_import, terminal::alacritty_import(rice)),
    });
    Ok(files)
}

/// Reads a file for [`plan`]: missing files count as empty.
pub fn read_or_empty(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_default()
}

/// Writes the planned files. An existing file is first copied to `<name>.kazari.bak`
/// (only once, so the backup keeps the user's original config).
pub fn write(files: &[PlannedFile]) -> Result<(), PlanError> {
    for planned in files {
        let io_err = |source| PlanError::Io {
            path: planned.path.clone(),
            source,
        };
        if let Some(parent) = planned.path.parent() {
            fs::create_dir_all(parent).map_err(io_err)?;
        }
        if planned.path.exists() {
            let mut backup = planned.path.clone().into_os_string();
            backup.push(".kazari.bak");
            let backup = PathBuf::from(backup);
            if !backup.exists() {
                fs::copy(&planned.path, &backup).map_err(io_err)?;
            }
        }
        fs::write(&planned.path, &planned.contents).map_err(io_err)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::sample_rice;

    fn scratch_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("kazari-test-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn plan_only_includes_the_selected_terminal() {
        let paths = Paths::export(Path::new("out"));
        let files = plan(&sample_rice(), &paths, |_| String::new()).unwrap();
        let names: Vec<_> = files.iter().map(|f| f.path.clone()).collect();
        assert!(names.contains(&paths.windows_terminal));
        assert!(!names.contains(&paths.wezterm_module));
        assert_eq!(files.len(), 5);
    }

    #[test]
    fn plan_reports_which_file_is_broken() {
        let paths = Paths::export(Path::new("out"));
        let err = plan(&sample_rice(), &paths, |p| {
            if p == paths.komorebi {
                "{ nope".into()
            } else {
                String::new()
            }
        })
        .unwrap_err();
        assert!(err.to_string().contains("komorebi.json"));
    }

    #[test]
    fn write_backs_up_the_original_once() {
        let dir = scratch_dir("backup");
        let target = dir.join("komorebi.json");
        fs::create_dir_all(&dir).unwrap();
        fs::write(&target, "original").unwrap();

        let planned = |contents: &str| {
            vec![PlannedFile {
                path: target.clone(),
                contents: contents.into(),
            }]
        };
        write(&planned("first")).unwrap();
        write(&planned("second")).unwrap();

        assert_eq!(fs::read_to_string(&target).unwrap(), "second");
        assert_eq!(
            fs::read_to_string(dir.join("komorebi.json.kazari.bak")).unwrap(),
            "original"
        );
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn windows_paths_follow_the_usual_layout() {
        let paths = Paths::windows(
            Path::new("C:/Users/s"),
            Path::new("C:/Users/s/AppData/Local"),
            None,
        );
        assert_eq!(paths.komorebi, Path::new("C:/Users/s/komorebi.json"));
        assert!(paths.windows_terminal.ends_with("LocalState/settings.json"));
        let custom = Paths::windows(
            Path::new("C:/Users/s"),
            Path::new("L"),
            Some(Path::new("D:/komorebi")),
        );
        assert_eq!(custom.komorebi, Path::new("D:/komorebi/komorebi.json"));
    }
}
