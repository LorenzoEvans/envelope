use ratatui::prelude::*;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Modifier},
    widgets::{Block, Borders, List, ListItem, Paragraph, Clear},
    Frame,
};

use crate::app::{App, ActiveList, CurrentlyEditing};

pub fn render(app: &mut App, f: &mut Frame) {
    let size = f.size();

    // Define Theme Colors
    let bg_color = Color::Rgb(20, 10, 30); // Very dark purple
    let border_color = Color::Rgb(100, 50, 150); // Violet
    let highlight_color = Color::Rgb(180, 100, 240); // Bright violet
    let accent_color = Color::Rgb(130, 0, 130); // Dark Magenta
    let text_color = Color::Rgb(220, 200, 255); // Pale violet
    let warning_color = Color::Rgb(255, 50, 100); // Pinkish red

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Envelope - Environment Manager ")
        .title_alignment(Alignment::Center)
        .border_style(Style::default().fg(border_color))
        .bg(bg_color);
    f.render_widget(block, size);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Search bar
            Constraint::Min(10),   // Main lists
            Constraint::Length(6), // Editors
            Constraint::Length(3), // Help/Footer
        ].as_ref())
        .split(size);

    // 1. Search Bar
    let search_style = if app.searching {
        Style::default().fg(highlight_color).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(border_color)
    };
    let search_text = if app.search_query.is_empty() && !app.searching {
        " Press / to search... ".to_string()
    } else {
        format!(" {} ", app.search_query)
    };
    let search_bar = Paragraph::new(search_text)
        .block(Block::default().borders(Borders::ALL).title(" Search ").border_style(search_style))
        .style(Style::default().fg(text_color));
    f.render_widget(search_bar, chunks[0]);

    // 2. Main Lists
    let list_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    // Env List
    let env_border_style = if app.activated_list == ActiveList::EnvList {
        Style::default().fg(highlight_color).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(border_color)
    };
    let env_vars = app.filtered_env_vars();
    let env_items: Vec<ListItem> = env_vars
        .iter()
        .map(|(key, value)| {
            ListItem::new(format!(" {}: {}", key, value))
                .style(Style::default().fg(text_color))
        })
        .collect();
    let env_list = List::new(env_items)
        .block(Block::default().borders(Borders::ALL).title(" Environment Variables ").border_style(env_border_style))
        .highlight_symbol(" \u{25b6} ")
        .highlight_style(Style::default().bg(accent_color).fg(Color::White));
    f.render_stateful_widget(env_list, list_chunks[0], &mut app.env_list_state);

    // Path List
    let path_border_style = if app.activated_list == ActiveList::PathList {
        Style::default().fg(highlight_color).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(border_color)
    };
    let path_dirs = app.filtered_path_dirs();
    let path_items: Vec<ListItem> = path_dirs
        .iter()
        .map(|path| {
            ListItem::new(format!(" {:?}", path))
                .style(Style::default().fg(text_color))
        })
        .collect();
    let path_list = List::new(path_items)
        .block(Block::default().borders(Borders::ALL).title(" PATH Components ").border_style(path_border_style))
        .highlight_symbol(" \u{25b6} ")
        .highlight_style(Style::default().bg(accent_color).fg(Color::White));
    f.render_stateful_widget(path_list, list_chunks[1], &mut app.path_list_state);

    // 3. Editors
    let editor_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[2]);

    let env_edit_title = if app.editing && app.activated_list == ActiveList::EnvList { " Editing Value " } else { " Value " };
    let env_edit_val = if app.editing && app.activated_list == ActiveList::EnvList { app.env_var_value.as_str() } else { app.selected_value() };
    let env_editor = Paragraph::new(env_edit_val)
        .block(Block::default().borders(Borders::ALL).title(env_edit_title).border_style(env_border_style))
        .style(Style::default().fg(text_color));
    f.render_widget(env_editor, editor_chunks[0]);

    let path_edit_title = if app.editing && app.activated_list == ActiveList::PathList { " Editing Path " } else { " Selected Path " };
    let path_edit_val = if app.editing && app.activated_list == ActiveList::PathList { app.path_var_edit.as_str() } else { 
        if app.selected_path_dir < path_dirs.len() {
            path_dirs[app.selected_path_dir].to_str().unwrap_or("")
        } else {
            ""
        }
    };
    let path_editor = Paragraph::new(path_edit_val)
        .block(Block::default().borders(Borders::ALL).title(path_edit_title).border_style(path_border_style))
        .style(Style::default().fg(text_color));
    f.render_widget(path_editor, editor_chunks[1]);

    // 4. Footer
    let help_text = " Tab: Switch | /: Search | n: New | e: Edit | Enter: Save | q: Quit ";
    let footer = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(border_color)))
        .alignment(Alignment::Center)
        .style(Style::default().fg(text_color));
    f.render_widget(footer, chunks[3]);

    // Overlays
    if app.overwrite {
        render_overwrite_modal(f, warning_color, text_color);
    }

    if app.creating_new {
        render_new_var_modal(app, f, highlight_color, border_color, text_color);
    }
}

fn render_overwrite_modal(f: &mut Frame, warning_color: Color, text_color: Color) {
    let area = centered_rect(60, 20, f.size());
    f.render_widget(Clear, area);
    let block = Block::default()
        .title(" CONFIRM OVERWRITE ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(warning_color).add_modifier(Modifier::BOLD))
        .bg(Color::Rgb(40, 0, 10));
    
    let text = vec![
        Line::from("This variable already exists in your shell config."),
        Line::from("Do you want to overwrite it with the new value?"),
        Line::from(""),
        Line::from(Span::styled(" (y) Yes  /  (n) No ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
    ];
    let paragraph = Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Center)
        .style(Style::default().fg(text_color));
    
    f.render_widget(paragraph, area);
}

fn render_new_var_modal(app: &App, f: &mut Frame, highlight_color: Color, border_color: Color, text_color: Color) {
    let area = centered_rect(70, 40, f.size());
    f.render_widget(Clear, area);
    let block = Block::default()
        .title(" CREATE NEW VARIABLE ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(highlight_color).add_modifier(Modifier::BOLD))
        .bg(Color::Rgb(20, 10, 40));
    
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(1),
        ])
        .split(area);

    let name_style = if app.currently_editing == Some(CurrentlyEditing::EnvVarName) {
        Style::default().fg(highlight_color)
    } else {
        Style::default().fg(border_color)
    };
    let name_input = Paragraph::new(app.env_var_key.as_str())
        .block(Block::default().borders(Borders::ALL).title(" Variable Name ").border_style(name_style))
        .style(Style::default().fg(text_color));
    f.render_widget(name_input, chunks[0]);

    let val_style = if app.currently_editing == Some(CurrentlyEditing::EnvVarValue) {
        Style::default().fg(highlight_color)
    } else {
        Style::default().fg(border_color)
    };
    let val_input = Paragraph::new(app.env_var_value.as_str())
        .block(Block::default().borders(Borders::ALL).title(" Variable Value ").border_style(val_style))
        .style(Style::default().fg(text_color));
    f.render_widget(val_input, chunks[1]);

    let footer = Paragraph::new(" Enter: Next/Save | Esc: Cancel ")
        .alignment(Alignment::Center)
        .style(Style::default().fg(border_color));
    f.render_widget(footer, chunks[2]);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
