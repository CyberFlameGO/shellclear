use crate::shell;
use anyhow::anyhow;
use anyhow::Result;
use log::debug;
use std::fs;
use std::fs::write;
use std::path::Path;
extern crate chrono;
use chrono::{DateTime, Local};

/// timestemp format for attache backups file.
const DATE_TIME_BACKUP_FORMAT: &str = "%Y%m%d%H%M%S%.f";
/// the base folder name for the crates to store the data like backups, pop files etc.
const STATE_FOLDER_NAME: &str = ".shellclear";
/// backup folder name
const BACKUP_FOLDER: &str = "backups";
/// stash folder name
const STASH_FOLDER: &str = "stash";

/// describe the shell context which contain the app folder path and the shell history details
#[derive(Clone, Debug)]
pub struct ShellContext {
    app_folder_path: String,
    pub history: shell::HistoryShell,
}

/// Init crates state folder for storing history data and detect all history shell files
pub fn init() -> Result<Vec<ShellContext>> {
    let homedir = match home::home_dir() {
        Some(h) => h.display().to_string(),
        None => return Err(anyhow!("could not get directory path")),
    };

    // create a application folder to save all the temp data
    let state_folder = Path::new(&homedir)
        .join(STATE_FOLDER_NAME)
        .display()
        .to_string();

    if let Err(err) = fs::create_dir(&state_folder) {
        if err.kind() != std::io::ErrorKind::AlreadyExists {
            return Err(anyhow!("could not create folder: {}", err));
        }
        debug!("state folder found: {:?}", state_folder);
    } else {
        debug!("state created in path: {:?}", state_folder);
    }

    Ok(shell::get_all_history_files()?
        .iter()
        .map(|h| ShellContext {
            app_folder_path: state_folder.clone(),
            history: h.clone(),
        })
        .collect::<Vec<_>>())
}

impl ShellContext {
    /// backup history shell file to backup folder
    pub fn backup(&self) -> Result<String> {
        let datetime: DateTime<Local> = Local::now().into();

        let copy_to = Path::new(&self.get_backup_folder())
            .join(format!(
                "{}.{}.bak",
                self.history.file_name,
                datetime.format(DATE_TIME_BACKUP_FORMAT)
            ))
            .display()
            .to_string();

        // create backup folder
        debug!("backup file: {} to: {}", &self.history.path, copy_to);
        fs::create_dir_all(self.get_backup_folder())?;
        debug!(
            "backup successfully file: {} to: {}",
            &self.history.path, copy_to
        );
        fs::copy(&self.history.path, &copy_to)?;
        Ok(copy_to)
    }

    /// restore the given history file path
    pub fn restore(&self, file_path: &str) -> Result<()> {
        debug!("restore file: {} to: {}", file_path, &self.history.path);
        fs::copy(file_path, &self.history.path)?;
        debug!(
            "restore successfully file: {} to: {}",
            file_path, &self.history.path
        );
        Ok(())
    }

    /// override history content with new content
    pub fn save_history_content(&self, content: &str) -> Result<()> {
        debug!("save history content in path: {:}", &self.history.path);
        write(&self.history.path, content)?;
        debug!(
            "save successfully history content in path: {:}",
            &self.history.path
        );
        Ok(())
    }

    /// save history file in stash folder clear current history shell file
    pub fn stash(&self) -> Result<String> {
        let copy_to = Path::new(&self.get_stash_folder())
            .join(&self.history.file_name)
            .display()
            .to_string();

        if !Path::new(&self.history.path).exists() {
            debug!("history path not found: {}", self.history.path);
            return Err(anyhow!("history path {} not found", self.history.path));
        }
        // create backup folder
        fs::create_dir_all(self.get_stash_folder())?;
        debug!("stash file: {} to: {}", &self.history.path, copy_to);
        fs::copy(&self.history.path, &copy_to)?;
        debug!("remove file: {}", self.history.path);
        fs::remove_file(&self.history.path)?;
        debug!("remove successfully file: {}", self.history.path);
        Ok(copy_to)
    }

    /// move stash history file to current shell history file
    pub fn pop(&self) -> Result<String> {
        let copy_from = Path::new(&self.get_stash_folder())
            .join(&self.history.file_name)
            .display()
            .to_string();

        debug!("pop file: {} to: {}", &copy_from, &self.history.path);
        fs::copy(&copy_from, &self.history.path)?;
        fs::remove_file(&copy_from)?;
        debug!("remove successfully file: {}", copy_from);
        Ok(self.history.path.to_string())
    }

    /// return all backup files
    pub fn get_backup_files(&self) -> Result<Vec<String>> {
        let backup_folder = self.get_backup_folder();
        if !Path::new(&backup_folder).is_dir() {
            return Ok(vec![]);
        }
        let paths = fs::read_dir(backup_folder)?;
        Ok(paths
            .map(|path| path.unwrap().path().display().to_string())
            .collect::<Vec<_>>())
    }

    /// return stash file
    pub fn get_stash_file(&self) -> Option<String> {
        let path = Path::new(&self.get_stash_folder()).join(&self.history.file_name);
        if path.exists() {
            Some(path.display().to_string())
        } else {
            None
        }
    }

    /// check if stash file exists
    pub fn is_stash_file_exists(&self) -> Result<bool> {
        let stash_folder = self.get_stash_folder();
        if !Path::new(&stash_folder).is_dir() {
            return Ok(false);
        }
        let paths = fs::read_dir(&stash_folder)?;
        let count = paths
            .filter(|path| {
                path.as_ref()
                    .unwrap()
                    .file_name()
                    .to_str()
                    .unwrap()
                    .to_string()
                    == self.history.file_name
            })
            .count();
        debug!("found {} stash files in {}", &stash_folder, &count);
        Ok(count >= 1)
    }

    /// return a backup folder path
    fn get_backup_folder(&self) -> String {
        Path::new(&self.app_folder_path)
            .join(BACKUP_FOLDER)
            .display()
            .to_string()
    }

    /// return stash folder
    fn get_stash_folder(&self) -> String {
        Path::new(&self.app_folder_path)
            .join(STASH_FOLDER)
            .display()
            .to_string()
    }
}

#[cfg(test)]
mod state_context {
    use super::*;
    use crate::shell::Shell;
    use std::fs::File;
    use std::io::Write;
    use tempdir::TempDir;

    const TEMP_HISTORY_CONTENT: &str = "history
ls
echo 'hello you'
rm -f ./file.txt
export GITHUB_TOKEN=token
";

    fn create_mock_state(temp_dir: &TempDir) -> ShellContext {
        let app_folder = temp_dir.path().join("app");
        let history_file_name = "history";
        let history_file_path = app_folder.join(history_file_name);
        fs::create_dir_all(&app_folder).unwrap();

        let mut f = File::create(&history_file_path).unwrap();
        f.write_all(TEMP_HISTORY_CONTENT.as_bytes()).unwrap();
        f.sync_all().unwrap();

        ShellContext {
            app_folder_path: app_folder.display().to_string(),
            history: shell::HistoryShell {
                shell: Shell::Bash,
                path: history_file_path.display().to_string(),
                file_name: history_file_name.to_string(),
            },
        }
    }

    #[test]
    fn can_backup_file() {
        let temp_dir = TempDir::new("terminal").unwrap();
        let context = create_mock_state(&temp_dir);

        assert!(context.backup().is_ok());
        let backup_folder = context.get_backup_files().unwrap();
        assert_eq!(backup_folder.len(), 1);
        assert_eq!(
            fs::read_to_string(backup_folder[0].clone()).unwrap(),
            TEMP_HISTORY_CONTENT
        );
        temp_dir.close().unwrap();
    }

    #[test]
    fn can_restore_file() {
        let temp_dir = TempDir::new("terminal").unwrap();
        let restore_file = temp_dir.path().join("backup-history-file");
        let context = create_mock_state(&temp_dir);

        let mut f = File::create(&restore_file).unwrap();
        f.write_all("backup-history_file".as_bytes()).unwrap();
        f.sync_all().unwrap();

        assert!(context.restore(&restore_file.display().to_string()).is_ok());
        assert_eq!(
            fs::read_to_string(&context.history.path).unwrap(),
            "backup-history_file"
        );
        temp_dir.close().unwrap();
    }

    #[test]
    fn can_save_history_content() {
        let temp_dir = TempDir::new("terminal").unwrap();
        let context = create_mock_state(&temp_dir);

        // save_history_content

        assert!(context.save_history_content("new commands").is_ok());
        assert_eq!(
            fs::read_to_string(&context.history.path).unwrap(),
            "new commands"
        );
        temp_dir.close().unwrap();
    }

    #[test]
    fn can_stash() {
        let temp_dir = TempDir::new("terminal").unwrap();
        let context = create_mock_state(&temp_dir);

        assert!(Path::new(&context.history.path).exists());
        assert!(context.stash().is_ok());
        assert!(!Path::new(&context.history.path).exists());
        assert_eq!(fs::read_dir(context.get_stash_folder()).unwrap().count(), 1);
        temp_dir.close().unwrap();
    }

    #[test]
    fn can_find_stash_file() {
        let temp_dir = TempDir::new("terminal").unwrap();
        let context = create_mock_state(&temp_dir);

        assert!(context.stash().is_ok());
        assert!(context.is_stash_file_exists().unwrap());
        temp_dir.close().unwrap();
    }

    #[test]
    fn can_pop() {
        let temp_dir = TempDir::new("terminal").unwrap();
        let context = create_mock_state(&temp_dir);

        assert_eq!(
            fs::read_to_string(&context.history.path).unwrap(),
            TEMP_HISTORY_CONTENT
        );
        assert!(context.stash().is_ok());
        assert!(!Path::new(&context.history.path).exists());
        assert!(context.pop().is_ok());
        assert!(Path::new(&context.history.path).exists());
        assert_eq!(
            fs::read_to_string(&context.history.path).unwrap(),
            TEMP_HISTORY_CONTENT
        );

        temp_dir.close().unwrap();
    }

    #[test]
    fn can_get_backup_files() {
        let temp_dir = TempDir::new("terminal").unwrap();
        let context = create_mock_state(&temp_dir);

        context.backup().unwrap();
        context.backup().unwrap();
        context.backup().unwrap();

        let backup_folder = context.get_backup_files().unwrap();
        assert_eq!(backup_folder.len(), 3);
        temp_dir.close().unwrap();
    }

    #[test]
    fn can_get_stash_file() {
        let temp_dir = TempDir::new("terminal").unwrap();
        let context = create_mock_state(&temp_dir);

        assert!(&context.stash().is_ok());
        assert_eq!(
            Path::new(&context.get_stash_folder())
                .join(&context.history.file_name)
                .display()
                .to_string(),
            context.get_stash_file().unwrap()
        );
    }
}
