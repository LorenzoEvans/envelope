use crate::app::{ActiveList, App, AppResult, CurrentlyEditing};
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::fs::{create_dir_all, read_to_string, rename, write};
use std::path::{Path, PathBuf};

pub fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    if app.searching {
        handle_search_events(key_event, app)?;
        return Ok(());
    }

    if app.overwrite {
        handle_overwrite_events(key_event, app)?;
        return Ok(());
    }

    if app.creating_new {
        handle_new_var_events(key_event, app)?;
        return Ok(());
    }

    match key_event.code {
        KeyCode::Char('q') | KeyCode::Esc => {
            if app.editing {
                app.editing = false;
            } else {
                app.quit();
            }
        }
        KeyCode::Char('c') | KeyCode::Char('C') if key_event.modifiers == KeyModifiers::CONTROL => {
            app.quit();
        }
        KeyCode::Char('e') if !app.editing => {
            app.editing = true;
            match app.activated_list {
                ActiveList::EnvList => {
                    let filtered = app.filtered_env_vars();
                    if app.selected_env_var < filtered.len() {
                        app.env_var_value = filtered[app.selected_env_var].1.clone();
                    }
                }
                ActiveList::PathList => {
                    let filtered = app.filtered_path_dirs();
                    if app.selected_path_dir < filtered.len() {
                        app.path_var_edit = filtered[app.selected_path_dir]
                            .to_string_lossy()
                            .to_string();
                    }
                }
            }
        }
        KeyCode::Char('n') if !app.editing => {
            app.creating_new = true;
            app.env_var_key = String::new();
            app.env_var_value = String::new();
            app.currently_editing = Some(CurrentlyEditing::EnvVarName);
        }
        KeyCode::Char('/') if !app.editing => {
            app.searching = true;
        }
        KeyCode::Char(c) if app.editing => match app.activated_list {
            ActiveList::EnvList => app.env_var_value.push(c),
            ActiveList::PathList => {
                app.path_var_edit.push(c);
            }
        },
        KeyCode::Backspace if app.editing => match app.activated_list {
            ActiveList::EnvList => {
                app.env_var_value.pop();
            }
            ActiveList::PathList => {
                app.path_var_edit.pop();
            }
        },
        KeyCode::Tab if !app.editing => {
            app.toggle_active();
        }
        KeyCode::Down => match app.activated_list {
            ActiveList::EnvList => {
                let filtered = app.filtered_env_vars();
                if !app.editing && !filtered.is_empty() && app.selected_env_var < filtered.len() - 1
                {
                    app.selected_env_var += 1;
                    app.env_list_state.select(Some(app.selected_env_var))
                }
            }
            ActiveList::PathList => {
                let filtered = app.filtered_path_dirs();
                if !app.editing
                    && !filtered.is_empty()
                    && app.selected_path_dir < filtered.len() - 1
                {
                    app.selected_path_dir += 1;
                    app.path_list_state.select(Some(app.selected_path_dir))
                }
            }
        },
        KeyCode::Up => match app.activated_list {
            ActiveList::EnvList => {
                if !app.editing && app.selected_env_var > 0 {
                    app.selected_env_var -= 1;
                    app.env_list_state.select(Some(app.selected_env_var))
                }
            }
            ActiveList::PathList => {
                if !app.editing && app.selected_path_dir > 0 {
                    app.selected_path_dir -= 1;
                    app.path_list_state.select(Some(app.selected_path_dir))
                }
            }
        },
        KeyCode::Enter if app.editing => match app.activated_list {
            ActiveList::EnvList => {
                let filtered = app.filtered_env_vars();
                if app.selected_env_var < filtered.len() {
                    let key = filtered[app.selected_env_var].0.clone();
                    if app.shell_env_vars.contains_key(&key) {
                        app.overwrite = true;
                    } else {
                        match save_env_var(app, key, app.env_var_value.clone()) {
                            Ok(()) => {
                                app.editing = false;
                                app.set_status("Environment variable saved");
                            }
                            Err(err) => app.set_error(err.to_string()),
                        }
                    }
                }
            }
            ActiveList::PathList => {
                let filtered = app.filtered_path_dirs();
                if app.selected_path_dir < filtered.len() {
                    let new_path = PathBuf::from(app.path_var_edit.clone());
                    match save_path_var(app, new_path) {
                        Ok(()) => {
                            app.editing = false;
                            app.set_status("PATH entry saved");
                        }
                        Err(err) => app.set_error(err.to_string()),
                    }
                }
            }
        },
        _ => {}
    }
    Ok(())
}

fn handle_search_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    match key_event.code {
        KeyCode::Enter | KeyCode::Esc => {
            app.searching = false;
        }
        KeyCode::Char(c) => {
            app.search_query.push(c);
            app.selected_env_var = 0;
            app.selected_path_dir = 0;
            app.env_list_state.select(Some(0));
            app.path_list_state.select(Some(0));
        }
        KeyCode::Backspace => {
            app.search_query.pop();
            app.selected_env_var = 0;
            app.selected_path_dir = 0;
            app.env_list_state.select(Some(0));
            app.path_list_state.select(Some(0));
        }
        _ => {}
    }
    Ok(())
}

fn handle_overwrite_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    match key_event.code {
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            let filtered = app.filtered_env_vars();
            if app.selected_env_var < filtered.len() {
                let key = filtered[app.selected_env_var].0.clone();
                if let Err(err) = save_env_var(app, key, app.env_var_value.clone()) {
                    app.set_error(err.to_string());
                } else {
                    app.set_status("Environment variable saved");
                }
            }
            app.overwrite = false;
            app.editing = false;
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            app.overwrite = false;
        }
        _ => {}
    }
    Ok(())
}

fn handle_new_var_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    match key_event.code {
        KeyCode::Esc => {
            app.creating_new = false;
            app.currently_editing = None;
        }
        KeyCode::Enter => match app.currently_editing {
            Some(CurrentlyEditing::EnvVarName) if !app.env_var_key.is_empty() => {
                app.currently_editing = Some(CurrentlyEditing::EnvVarValue);
            }
            Some(CurrentlyEditing::EnvVarValue) => {
                match save_env_var(app, app.env_var_key.clone(), app.env_var_value.clone()) {
                    Ok(()) => {
                        app.creating_new = false;
                        app.currently_editing = None;
                        app.set_status("Environment variable created");
                    }
                    Err(err) => app.set_error(err.to_string()),
                }
            }
            _ => {}
        },
        KeyCode::Char(c) => match app.currently_editing {
            Some(CurrentlyEditing::EnvVarName) => app.env_var_key.push(c),
            Some(CurrentlyEditing::EnvVarValue) => app.env_var_value.push(c),
            _ => {}
        },
        KeyCode::Backspace => match app.currently_editing {
            Some(CurrentlyEditing::EnvVarName) => {
                app.env_var_key.pop();
            }
            Some(CurrentlyEditing::EnvVarValue) => {
                app.env_var_value.pop();
            }
            _ => {}
        },
        _ => {}
    }
    Ok(())
}

fn save_env_var(app: &mut App, key: String, value: String) -> AppResult<()> {
    if !is_valid_env_var_name(&key) {
        return Err(format!("invalid environment variable name: {key}").into());
    }

    upsert_shell_assignment(app, &key, &value)?;

    if let Some(pos) = app.env_vars.iter().position(|(k, _)| k == &key) {
        app.env_vars[pos].1 = value.clone();
    } else {
        app.env_vars.push((key.clone(), value.clone()));
    }
    app.shell_env_vars.insert(key, value);

    Ok(())
}

fn save_path_var(app: &mut App, new_path: PathBuf) -> AppResult<()> {
    let filtered = app.filtered_path_dirs();
    let old_path = filtered.get(app.selected_path_dir).cloned();

    append_path_assignment(app, &new_path)?;

    if let Some(old_path) = old_path {
        if let Some(pos) = app.path_var_dirs.iter().position(|p| p == &old_path) {
            app.path_var_dirs[pos] = new_path.clone();
        }
    }

    Ok(())
}

fn upsert_shell_assignment(app: &App, key: &str, value: &str) -> AppResult<()> {
    let content = read_to_string(&app.config_path).unwrap_or_default();
    let assignment = format_assignment(&app.shell, key, value);
    let mut replaced = false;
    let mut lines = Vec::new();

    for line in content.lines() {
        if line_assigns_key(line, key) {
            if !replaced {
                lines.push(assignment.clone());
                replaced = true;
            }
        } else {
            lines.push(line.to_string());
        }
    }

    if !replaced {
        if !lines.is_empty() {
            lines.push(String::new());
        }
        lines.push(assignment);
    }

    write_config(app, &finish_lines(lines))?;
    Ok(())
}

fn append_path_assignment(app: &App, path: &Path) -> AppResult<()> {
    let mut content = read_to_string(&app.config_path).unwrap_or_default();
    if !content.is_empty() && !content.ends_with('\n') {
        content.push('\n');
    }
    if !content.is_empty() {
        content.push('\n');
    }
    content.push_str(&format_path_assignment(&app.shell, &path.to_string_lossy()));
    content.push('\n');
    write_config(app, &content)?;
    Ok(())
}

fn write_config(app: &App, content: &str) -> AppResult<()> {
    if let Some(parent) = app
        .config_path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
    {
        create_dir_all(parent)?;
    }

    let temp_path = app
        .config_path
        .with_extension(format!("envelope-{}.tmp", std::process::id()));
    write(&temp_path, content)?;
    if let Err(err) = rename(&temp_path, &app.config_path) {
        let _ = std::fs::remove_file(&temp_path);
        return Err(err.into());
    }
    Ok(())
}

fn is_valid_env_var_name(name: &str) -> bool {
    let mut chars = name.chars();
    matches!(chars.next(), Some('_' | 'A'..='Z' | 'a'..='z'))
        && chars.all(|c| matches!(c, '_' | 'A'..='Z' | 'a'..='z' | '0'..='9'))
}

fn format_assignment(shell_config: &str, key: &str, value: &str) -> String {
    if shell_config == "config.fish" {
        format!("set -gx {key} {}", shell_quote(value))
    } else {
        format!("export {key}={}", shell_quote(value))
    }
}

fn format_path_assignment(shell_config: &str, path: &str) -> String {
    if shell_config == "config.fish" {
        format!("fish_add_path {}", shell_quote(path))
    } else {
        format!("export PATH=\"$PATH\":{}", shell_quote(path))
    }
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn line_assigns_key(line: &str, key: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed
        .strip_prefix("export ")
        .is_some_and(|rest| rest.trim_start().starts_with(&format!("{key}=")))
        || trimmed
            .strip_prefix("set -gx ")
            .is_some_and(|rest| rest.trim_start().starts_with(&format!("{key} ")))
}

fn finish_lines(lines: Vec<String>) -> String {
    if lines.is_empty() {
        String::new()
    } else {
        format!("{}\n", lines.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::App;
    use std::fs::{read_to_string, write};
    use tempfile::TempDir;

    fn create_test_app() -> (App, TempDir) {
        let temp_dir = tempfile::tempdir().unwrap();
        let app = App {
            config_path: temp_dir.path().join(".test_config"),
            shell: ".bashrc".to_string(),
            ..Default::default()
        };
        (app, temp_dir)
    }

    #[test]
    fn test_save_env_var() {
        let (mut app, _temp_dir) = create_test_app();
        let key = "TEST_VAR".to_string();
        let value = "test_value".to_string();

        save_env_var(&mut app, key.clone(), value.clone()).unwrap();

        assert_eq!(
            app.env_vars.iter().find(|(k, _)| k == &key).unwrap().1,
            value
        );

        let contents = read_to_string(&app.config_path).unwrap();
        assert!(contents.contains("export TEST_VAR='test_value'"));
    }

    #[test]
    fn test_save_path_var() {
        let (mut app, _temp_dir) = create_test_app();
        app.path_var_dirs = vec![PathBuf::from("/usr/bin")];
        app.selected_path_dir = 0;
        let new_path = PathBuf::from("/new/path");

        save_path_var(&mut app, new_path.clone()).unwrap();

        assert_eq!(app.path_var_dirs[0], new_path);

        let contents = read_to_string(&app.config_path).unwrap();
        assert!(contents.contains("export PATH=\"$PATH\":'/new/path'"));
    }

    #[test]
    fn test_save_env_var_replaces_existing_assignment() {
        let (mut app, _temp_dir) = create_test_app();
        write(
            &app.config_path,
            "export TEST_VAR='old'\nexport OTHER='ok'\n",
        )
        .unwrap();

        save_env_var(&mut app, "TEST_VAR".to_string(), "new".to_string()).unwrap();

        let contents = read_to_string(&app.config_path).unwrap();
        assert!(contents.contains("export TEST_VAR='new'"));
        assert!(contents.contains("export OTHER='ok'"));
        assert!(!contents.contains("export TEST_VAR='old'"));
        assert_eq!(contents.matches("export TEST_VAR=").count(), 1);
    }

    #[test]
    fn test_save_env_var_rejects_invalid_name() {
        let (mut app, _temp_dir) = create_test_app();

        let result = save_env_var(&mut app, "1_BAD".to_string(), "value".to_string());

        assert!(result.is_err());
        assert!(!app.config_path.exists());
    }

    #[test]
    fn test_save_env_var_shell_quotes_single_quotes() {
        let (mut app, _temp_dir) = create_test_app();

        save_env_var(&mut app, "QUOTED".to_string(), "don't split".to_string()).unwrap();

        let contents = read_to_string(&app.config_path).unwrap();
        assert!(contents.contains("export QUOTED='don'\\''t split'"));
    }

    #[test]
    fn test_save_env_var_uses_fish_assignment() {
        let (mut app, _temp_dir) = create_test_app();
        app.shell = "config.fish".to_string();

        save_env_var(&mut app, "FISH_VAR".to_string(), "value".to_string()).unwrap();

        let contents = read_to_string(&app.config_path).unwrap();
        assert!(contents.contains("set -gx FISH_VAR 'value'"));
    }

    #[test]
    fn test_save_env_var_creates_missing_parent_directory() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("nested").join("config");
        let mut app = App {
            config_path,
            shell: ".bashrc".to_string(),
            ..Default::default()
        };

        save_env_var(&mut app, "NESTED".to_string(), "value".to_string()).unwrap();

        assert!(app.config_path.exists());
    }

    #[test]
    fn test_save_env_var_does_not_update_memory_when_write_fails() {
        let temp_dir = tempfile::tempdir().unwrap();
        let mut app = App {
            config_path: temp_dir.path().to_path_buf(),
            shell: ".bashrc".to_string(),
            ..Default::default()
        };

        assert!(save_env_var(&mut app, "FAILED".to_string(), "value".to_string()).is_err());
        assert!(!app.env_vars.iter().any(|(key, _)| key == "FAILED"));
        assert!(!app.shell_env_vars.contains_key("FAILED"));
    }
}
