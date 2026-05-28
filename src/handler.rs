use crate::app::{ActiveList, App, AppResult, CurrentlyEditing};
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

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
        KeyCode::Char('c') | KeyCode::Char('C') => {
            if key_event.modifiers == KeyModifiers::CONTROL {
                app.quit();
            }
        }
        KeyCode::Char('e') => {
            if !app.editing {
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
                            app.path_var_edit = filtered[app.selected_path_dir].to_string_lossy().to_string();
                        }
                    }
                }
            }
        }
        KeyCode::Char('n') => {
            if !app.editing {
                app.creating_new = true;
                app.env_var_key = String::new();
                app.env_var_value = String::new();
                app.currently_editing = Some(CurrentlyEditing::EnvVarName);
            }
        }
        KeyCode::Char('/') => {
            if !app.editing {
                app.searching = true;
            }
        }
        KeyCode::Char(c) => {
            if app.editing {
                match app.activated_list {
                    ActiveList::EnvList => app.env_var_value.push(c),
                    ActiveList::PathList => {
                        app.path_var_edit.push(c);
                    }
                }
            }
        }
        KeyCode::Backspace => {
            if app.editing {
                match app.activated_list {
                    ActiveList::EnvList => {
                        app.env_var_value.pop();
                    }
                    ActiveList::PathList => {
                        app.path_var_edit.pop();
                    }
                }
            }
        }
        KeyCode::Tab => {
            if !app.editing {
                app.toggle_active();
            }
        }
        KeyCode::Down => match app.activated_list {
            ActiveList::EnvList => {
                let filtered = app.filtered_env_vars();
                if !app.editing && !filtered.is_empty() && app.selected_env_var < filtered.len() - 1 {
                    app.selected_env_var += 1;
                    app.env_list_state.select(Some(app.selected_env_var))
                }
            }
            ActiveList::PathList => {
                let filtered = app.filtered_path_dirs();
                if !app.editing && !filtered.is_empty() && app.selected_path_dir < filtered.len() - 1 {
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
        KeyCode::Enter => {
            if app.editing {
                match app.activated_list {
                    ActiveList::EnvList => {
                        let filtered = app.filtered_env_vars();
                        if app.selected_env_var < filtered.len() {
                            let key = filtered[app.selected_env_var].0.clone();
                            if app.shell_env_vars.contains_key(&key) {
                                app.overwrite = true;
                            } else {
                                save_env_var(app, key, app.env_var_value.clone())?;
                                app.editing = false;
                            }
                        }
                    }
                    ActiveList::PathList => {
                        let filtered = app.filtered_path_dirs();
                        if app.selected_path_dir < filtered.len() {
                            let new_path = PathBuf::from(app.path_var_edit.clone());
                            save_path_var(app, new_path)?;
                            app.editing = false;
                        }
                    }
                }
            }
        }
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
                save_env_var(app, key, app.env_var_value.clone())?;
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
        KeyCode::Enter => {
            match app.currently_editing {
                Some(CurrentlyEditing::EnvVarName) => {
                    if !app.env_var_key.is_empty() {
                        app.currently_editing = Some(CurrentlyEditing::EnvVarValue);
                    }
                }
                Some(CurrentlyEditing::EnvVarValue) => {
                    save_env_var(app, app.env_var_key.clone(), app.env_var_value.clone())?;
                    app.creating_new = false;
                    app.currently_editing = None;
                }
                _ => {}
            }
        }
        KeyCode::Char(c) => {
            match app.currently_editing {
                Some(CurrentlyEditing::EnvVarName) => app.env_var_key.push(c),
                Some(CurrentlyEditing::EnvVarValue) => app.env_var_value.push(c),
                _ => {}
            }
        }
        KeyCode::Backspace => {
            match app.currently_editing {
                Some(CurrentlyEditing::EnvVarName) => {
                    app.env_var_key.pop();
                }
                Some(CurrentlyEditing::EnvVarValue) => {
                    app.env_var_value.pop();
                }
                _ => {}
            }
        }
        _ => {}
    }
    Ok(())
}

fn save_env_var(app: &mut App, key: String, value: String) -> AppResult<()> {
    // Update local state
    if let Some(pos) = app.env_vars.iter().position(|(k, _)| k == &key) {
        app.env_vars[pos].1 = value.clone();
    } else {
        app.env_vars.push((key.clone(), value.clone()));
    }
    app.shell_env_vars.insert(key.clone(), value.clone());

    // Write to config
    let mut shell_config = OpenOptions::new().append(true).open(&app.config_path)?;
    let export_var = format!("export {}=\"{}\"\n", key, value);
    shell_config.write_all(b"\n")?;
    shell_config.write_all(export_var.as_bytes())?;
    
    Ok(())
}

fn save_path_var(app: &mut App, new_path: PathBuf) -> AppResult<()> {
    // Update local state
    let filtered = app.filtered_path_dirs();
    if app.selected_path_dir < filtered.len() {
        let old_path = filtered[app.selected_path_dir].clone();
        if let Some(pos) = app.path_var_dirs.iter().position(|p| p == &old_path) {
            app.path_var_dirs[pos] = new_path.clone();
        }
    }

    // Write to config
    let mut shell_config = OpenOptions::new().append(true).open(&app.config_path)?;
    let export_var = format!("export PATH=$PATH:{}\n", new_path.to_string_lossy());
    shell_config.write_all(b"\n")?;
    shell_config.write_all(export_var.as_bytes())?;

    Ok(())
}

pub fn write_to_config(_app: &App, _config_var: &str, _config_file: &mut File) {
    // This is now handled in save_env_var and save_path_var
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{File, remove_file};
    use std::io::{Read, Seek, SeekFrom};
    use crate::app::App;

    fn create_test_app() -> App {
        let mut app = App::default();
        let temp_file = ".test_config";
        File::create(temp_file).unwrap();
        app.config_path = PathBuf::from(temp_file);
        app
    }

    #[test]
    fn test_save_env_var() {
        let mut app = create_test_app();
        let key = "TEST_VAR".to_string();
        let value = "test_value".to_string();

        save_env_var(&mut app, key.clone(), value.clone()).unwrap();

        assert_eq!(app.env_vars.iter().find(|(k, _)| k == &key).unwrap().1, value);
        
        let mut file = File::open(&app.config_path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        assert!(contents.contains("export TEST_VAR=\"test_value\""));

        remove_file(&app.config_path).unwrap();
    }

    #[test]
    fn test_save_path_var() {
        let mut app = create_test_app();
        app.path_var_dirs = vec![PathBuf::from("/usr/bin")];
        app.selected_path_dir = 0;
        let new_path = PathBuf::from("/new/path");

        save_path_var(&mut app, new_path.clone()).unwrap();

        assert_eq!(app.path_var_dirs[0], new_path);

        let mut file = File::open(&app.config_path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        assert!(contents.contains("export PATH=$PATH:/new/path"));

        remove_file(&app.config_path).unwrap();
    }
}
