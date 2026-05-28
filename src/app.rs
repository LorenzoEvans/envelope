use std::collections::HashMap;
use std::env;
use std::env::{split_paths, var_os};
use std::error;
use std::fs::read_to_string;
use std::path::PathBuf;
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

pub struct App {
    /// Houses environment variables for the current environment.
    pub env_vars: Vec<(String, String)>,
    /// Houses the directories stored in the path variable.
    pub path_var_dirs: Vec<PathBuf>,
    /// Specifies which environment variable is currently being edited.
    pub selected_env_var: usize,
    /// Specifies the environment variable name associated with the value.
    pub selected_env_key: String,
    /// Specifies which path variable is currently being edited.
    pub selected_path_dir: usize,
    /// Specifies whether or not the app is in an `editing` state.
    pub editing: bool,
    /// Houses the edited environment variable value string.
    pub env_var_value: String,
    /// Houses the path variable being edited.
    pub path_var_value: String,
    pub path_var_edit: String,
    /// Houses the edited environment variable key string.
    pub env_var_key: String,
    /// Holds the state of the list of environment variables
    pub env_list_state: ratatui::widgets::ListState,
    /// Holds the state of the list of path variable components
    pub path_list_state: ratatui::widgets::ListState,
    /// Boolean to determine if app is running,
    pub running: bool,
    /// Houses the state indicating what a user is currently editing.
    pub currently_editing: Option<CurrentlyEditing>,
    /// Currently activated list number.
    pub list_index: u32,
    /// Currently active list widget.
    pub activated_list: ActiveList,
    /// User shell
    pub shell: String,
    /// Environment variables from .bashrc
    pub shell_env_vars: HashMap<String, String>,
    /// Shell config path
    pub config_path: PathBuf,
    /// Overwriting signifier
    pub overwrite: bool,
    /// Search query
    pub search_query: String,
    /// Is searching
    pub searching: bool,
    /// Is creating a new variable
    pub creating_new: bool,
}
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ActiveList {
    EnvList,
    PathList,
}

impl Default for App {
    fn default() -> App {
        let mut env_list_state = ratatui::widgets::ListState::default();
        env_list_state.select(Some(0));
        let mut path_list_state = ratatui::widgets::ListState::default();
        path_list_state.select(Some(0));
        let env_vars = env::vars().collect();
        let mut path_var_dirs = Vec::new();
        let shell = get_shell_config().unwrap_or_else(|_| ".bashrc".to_string());
        let config_path = get_config_path().unwrap_or_default();
        let key = "PATH";
        let path_var = var_os(key);
        let shell_env_vars = get_shell_vars().unwrap_or_default();

        match path_var {
            Some(paths) => {
                for path in split_paths(&paths) {
                    path_var_dirs.push(path);
                }
            }
            None => println!("{key} not set in current environment."),
        }
        App {
            env_vars,
            path_var_dirs,
            selected_env_var: 0,
            selected_env_key: String::new(),
            selected_path_dir: 0,
            editing: false,
            env_var_value: String::new(),
            env_var_key: String::new(),
            env_list_state,
            path_list_state,
            running: true,
            currently_editing: None,
            list_index: 0,
            activated_list: ActiveList::EnvList,
            path_var_value: String::new(),
            path_var_edit: String::new(),
            shell,
            shell_env_vars,
            config_path,
            overwrite: false,
            search_query: String::new(),
            searching: false,
            creating_new: false,
        }
    }
}
impl App {
    pub fn new() -> Self {
        App::default()
    }
    pub fn selected_value(&self) -> &str {
        if self.selected_env_var < self.env_vars.len() {
            &self.env_vars[self.selected_env_var].1
        } else {
            ""
        }
    }

    pub fn filtered_env_vars(&self) -> Vec<(String, String)> {
        if self.search_query.is_empty() {
            self.env_vars.clone()
        } else {
            self.env_vars
                .iter()
                .filter(|(k, v)| {
                    k.to_lowercase().contains(&self.search_query.to_lowercase())
                        || v.to_lowercase().contains(&self.search_query.to_lowercase())
                })
                .cloned()
                .collect()
        }
    }

    pub fn filtered_path_dirs(&self) -> Vec<PathBuf> {
        if self.search_query.is_empty() {
            self.path_var_dirs.clone()
        } else {
            self.path_var_dirs
                .iter()
                .filter(|p| {
                    p.to_string_lossy()
                        .to_lowercase()
                        .contains(&self.search_query.to_lowercase())
                })
                .cloned()
                .collect()
        }
    }

    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn toggle_active(&mut self) {
        match self.activated_list {
            ActiveList::EnvList => {
                self.activated_list = ActiveList::PathList;
                self.list_index = 1;
            }
            ActiveList::PathList => {
                self.activated_list = ActiveList::EnvList;
                self.list_index = 0;
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CurrentlyEditing {
    EnvVarValue,
    EnvVarName,
    PathVar,
}

// Reference code that may be deleted soon.

fn get_shell_config() -> Result<String, Box<dyn error::Error>> {
    if let Ok(shell_path) = std::env::var("SHELL") {
        let shell_name = shell_path.split('/').last().unwrap_or("");
        match shell_name {
            "bash" => return Ok(".bashrc".to_string()),
            "zsh" => return Ok(".zshrc".to_string()),
            "fish" => return Ok("config.fish".to_string()), // Simplified
            _ => {}
        }
    }

    let home = std::env::var("HOME")?;
    let home_path = std::path::PathBuf::from(home);
    
    let configs = [".bashrc", ".zshrc", ".bash_profile", ".profile"];
    for config in configs {
        if home_path.join(config).exists() {
            return Ok(config.to_string());
        }
    }

    Ok(".bashrc".to_string())
}

fn get_config_path() -> Result<PathBuf, Box<dyn error::Error>> {
    let shell = get_shell_config()?;
    let home = std::env::var("HOME")?;

    let mut home_path = PathBuf::from(home);
    if shell == "config.fish" {
        home_path.push(".config/fish/config.fish");
    } else {
        home_path.push(shell);
    }
    Ok(home_path)
}
pub fn get_shell_vars() -> Result<HashMap<String, String>, Box<dyn error::Error>> {
    let mut config_map = HashMap::new();
    let config_path = get_config_path()?;

    if !config_path.exists() {
        return Ok(config_map);
    }

    let config_file_contents = read_to_string(config_path)?;

    for line in config_file_contents.lines() {
        let line = line.trim();
        if line.starts_with("export") {
            if let Some(rest) = line.strip_prefix("export ") {
                if let Some((env_var_name, env_var_value)) = rest.split_once('=') {
                    let name = env_var_name.trim().to_owned();
                    let value = env_var_value.trim().trim_matches('\"').trim_matches('\'').to_owned();
                    config_map.insert(name, value);
                }
            }
        }
    }

    Ok(config_map)
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toggle_active_from_env_list() {
        let mut app = App::default();
        app.activated_list = ActiveList::EnvList;
        app.list_index = 0;

        app.toggle_active();

        assert_eq!(app.activated_list, ActiveList::PathList);
        assert_eq!(app.list_index, 1);
    }

    #[test]
    fn test_toggle_active_from_path_list() {
        let mut app = App::default();
        app.activated_list = ActiveList::PathList;
        app.list_index = 1;

        app.toggle_active();

        assert_eq!(app.activated_list, ActiveList::EnvList);
        assert_eq!(app.list_index, 0);
    }

    #[test]
    fn test_multiple_toggles() {
        let mut app = App::default();
        app.activated_list = ActiveList::EnvList;
        app.list_index = 0;

        app.toggle_active();
        app.toggle_active();

        assert_eq!(app.activated_list, ActiveList::EnvList);
        assert_eq!(app.list_index, 0);
    }

    #[test]
    fn test_toggle_active_updates_list_states() {
        let mut app = App::default();
        app.activated_list = ActiveList::EnvList;
        app.list_index = 0;
        app.env_list_state.select(Some(3));
        app.path_list_state.select(Some(2));

        app.toggle_active();

        assert_eq!(app.activated_list, ActiveList::PathList);
        assert_eq!(app.list_index, 1);
        assert_eq!(app.env_list_state.selected(), Some(3));
        assert_eq!(app.path_list_state.selected(), Some(2));

        app.toggle_active();

        assert_eq!(app.activated_list, ActiveList::EnvList);
        assert_eq!(app.list_index, 0);
        assert_eq!(app.env_list_state.selected(), Some(3));
        assert_eq!(app.path_list_state.selected(), Some(2));
    }
}
