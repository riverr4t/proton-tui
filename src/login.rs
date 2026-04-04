//! TUI-based login screen for Proton authentication.

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{block::BorderType, Block, Borders, Clear, Paragraph},
    Frame, Terminal,
};
use std::time::Duration;

use crate::config::AppConfig;
use crate::theme::Theme;

#[derive(PartialEq)]
enum LoginField {
    Username,
    Password,
}

pub struct LoginForm {
    username: String,
    password: String,
    focused_field: LoginField,
    cursor_position: usize,
    status_message: String,
    status_is_error: bool,
}

impl LoginForm {
    pub fn new() -> Self {
        Self {
            username: String::new(),
            password: String::new(),
            focused_field: LoginField::Username,
            cursor_position: 0,
            status_message: String::new(),
            status_is_error: false,
        }
    }

    pub fn set_status(&mut self, message: &str, is_error: bool) {
        self.status_message = message.to_string();
        self.status_is_error = is_error;
    }

    fn current_field(&self) -> &String {
        match self.focused_field {
            LoginField::Username => &self.username,
            LoginField::Password => &self.password,
        }
    }

    fn current_field_mut(&mut self) -> &mut String {
        match self.focused_field {
            LoginField::Username => &mut self.username,
            LoginField::Password => &mut self.password,
        }
    }

    fn switch_field(&mut self) {
        self.focused_field = match self.focused_field {
            LoginField::Username => LoginField::Password,
            LoginField::Password => LoginField::Username,
        };
        self.cursor_position = self.current_field().len();
    }

    fn handle_char(&mut self, c: char) {
        let pos = self.cursor_position;
        let field = self.current_field_mut();
        field.insert(pos, c);
        self.cursor_position += 1;
    }

    fn handle_backspace(&mut self) {
        if self.cursor_position > 0 {
            let pos = self.cursor_position - 1;
            let field = self.current_field_mut();
            field.remove(pos);
            self.cursor_position -= 1;
        }
    }

    fn handle_delete(&mut self) {
        let pos = self.cursor_position;
        let len = self.current_field().len();
        if pos < len {
            let field = self.current_field_mut();
            field.remove(pos);
        }
    }

    fn handle_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    fn handle_right(&mut self) {
        if self.cursor_position < self.current_field().len() {
            self.cursor_position += 1;
        }
    }

    fn handle_home(&mut self) {
        self.cursor_position = 0;
    }

    fn handle_end(&mut self) {
        self.cursor_position = self.current_field().len();
    }

    fn handle_clear_line(&mut self) {
        let field = self.current_field_mut();
        field.clear();
        self.cursor_position = 0;
    }
}

fn centered_rect(width: u16, height: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length((r.height.saturating_sub(height)) / 2),
            Constraint::Length(height),
            Constraint::Min(0),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length((r.width.saturating_sub(width)) / 2),
            Constraint::Length(width),
            Constraint::Min(0),
        ])
        .split(popup_layout[1])[1]
}

fn render_login(frame: &mut Frame, form: &LoginForm, theme: &Theme) {
    let t = theme;
    // Draw background
    let size = frame.size();
    let bg = Block::default().style(Style::default().bg(t.popup_bg));
    frame.render_widget(bg, size);

    // Login box
    let login_area = centered_rect(50, 14, size);

    frame.render_widget(Clear, login_area);

    let block = Block::default()
        .title(" 󰦝 ProtonVPN Login ")
        .title_style(Style::default().fg(t.accent).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .border_style(Style::default().fg(t.border_active))
        .style(Style::default().bg(t.popup_bg));

    frame.render_widget(block, login_area);

    let inner = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(1), // Title/info
            Constraint::Length(1), // Spacing
            Constraint::Length(3), // Username field
            Constraint::Length(3), // Password field
            Constraint::Length(1), // Status
            Constraint::Min(0),    // Hints
        ])
        .split(login_area);

    // Info text
    let info = Paragraph::new("Enter your Proton account credentials")
        .style(Style::default().fg(t.fg_dim))
        .alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(info, inner[0]);

    // Username field
    let username_focused = form.focused_field == LoginField::Username;
    let username_style = if username_focused {
        Style::default().fg(t.border_active)
    } else {
        Style::default()
    };
    let username_border_type = if username_focused {
        BorderType::Thick
    } else {
        BorderType::default()
    };

    let username_block = Block::default()
        .title(" Username ")
        .borders(Borders::ALL)
        .border_type(username_border_type)
        .border_style(username_style);

    let username_text = Paragraph::new(format!(" {}", form.username))
        .block(username_block)
        .style(Style::default().fg(t.fg));
    frame.render_widget(username_text, inner[2]);

    // Password field (masked)
    let password_focused = form.focused_field == LoginField::Password;
    let password_style = if password_focused {
        Style::default().fg(t.border_active)
    } else {
        Style::default()
    };
    let password_border_type = if password_focused {
        BorderType::Thick
    } else {
        BorderType::default()
    };

    let password_block = Block::default()
        .title(" Password ")
        .borders(Borders::ALL)
        .border_type(password_border_type)
        .border_style(password_style);

    let masked_password: String = "•".repeat(form.password.len());
    let password_text = Paragraph::new(format!(" {}", masked_password))
        .block(password_block)
        .style(Style::default().fg(t.fg));
    frame.render_widget(password_text, inner[3]);

    // Status message
    let status_style = if form.status_is_error {
        Style::default().fg(t.error)
    } else {
        Style::default().fg(t.warning)
    };
    let status = Paragraph::new(form.status_message.as_str())
        .style(status_style)
        .alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(status, inner[4]);

    // Hints (simple blue text style)
    let hint_style = Style::default().fg(t.info);
    let hint_bold = Style::default().fg(t.info).add_modifier(Modifier::BOLD);
    let sep_style = Style::default().fg(t.fg_muted);

    let hints = Line::from(vec![
        Span::styled("Tab", hint_bold),
        Span::styled(" Switch ", hint_style),
        Span::styled("| ", sep_style),
        Span::styled("Enter", hint_bold),
        Span::styled(" Login ", hint_style),
        Span::styled("| ", sep_style),
        Span::styled("Esc", hint_bold),
        Span::styled(" Quit", hint_style),
    ]);
    let hints_para = Paragraph::new(hints).alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(hints_para, inner[5]);

    // Set cursor position
    let (cursor_x, cursor_y) = match form.focused_field {
        LoginField::Username => (inner[2].x + 2 + form.cursor_position as u16, inner[2].y + 1),
        LoginField::Password => (inner[3].x + 2 + form.cursor_position as u16, inner[3].y + 1),
    };
    frame.set_cursor(cursor_x, cursor_y);
}

/// Result of the login form
pub enum LoginResult {
    /// User submitted credentials
    Submit { username: String, password: String },
    /// User cancelled (Esc or Ctrl+C)
    Cancel,
}

fn load_theme() -> Theme {
    AppConfig::load()
        .map(|c| Theme::from_name(&c.theme))
        .unwrap_or_default()
}

/// Run the login form and return the result
pub fn run_login<B: Backend>(terminal: &mut Terminal<B>) -> Result<LoginResult> {
    let mut form = LoginForm::new();
    let theme = load_theme();

    loop {
        terminal.draw(|f| render_login(f, &form, &theme))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                // Handle Ctrl+C
                if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                    return Ok(LoginResult::Cancel);
                }

                match key.code {
                    KeyCode::Esc => return Ok(LoginResult::Cancel),
                    KeyCode::Tab | KeyCode::Down | KeyCode::Up => form.switch_field(),
                    KeyCode::Enter => {
                        if form.username.is_empty() {
                            form.set_status("Username is required", true);
                            form.focused_field = LoginField::Username;
                            form.cursor_position = 0;
                        } else if form.password.is_empty() {
                            form.set_status("Password is required", true);
                            form.focused_field = LoginField::Password;
                            form.cursor_position = 0;
                        } else {
                            return Ok(LoginResult::Submit {
                                username: form.username,
                                password: form.password,
                            });
                        }
                    }
                    KeyCode::Backspace => form.handle_backspace(),
                    KeyCode::Delete => form.handle_delete(),
                    KeyCode::Left => form.handle_left(),
                    KeyCode::Right => form.handle_right(),
                    KeyCode::Home => form.handle_home(),
                    KeyCode::End => form.handle_end(),
                    KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        form.handle_home()
                    }
                    KeyCode::Char('e') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        form.handle_end()
                    }
                    KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        form.handle_clear_line()
                    }
                    KeyCode::Char(c) => form.handle_char(c),
                    _ => {}
                }

                // Clear error on typing
                if !matches!(key.code, KeyCode::Enter | KeyCode::Tab | KeyCode::Esc)
                    && form.status_is_error
                {
                    form.status_message.clear();
                    form.status_is_error = false;
                }
            }
        }
    }
}

/// Show authenticating status
pub fn show_authenticating<B: Backend>(terminal: &mut Terminal<B>) -> Result<()> {
    let mut form = LoginForm::new();
    let theme = load_theme();
    form.set_status("Authenticating... (Esc to cancel)", false);
    terminal.draw(|f| render_login(f, &form, &theme))?;
    Ok(())
}

/// Show a loading screen with a message
pub fn show_loading<B: Backend>(terminal: &mut Terminal<B>, message: &str) -> Result<()> {
    let theme = load_theme();
    terminal.draw(|f| {
        let t = &theme;
        let size = f.size();
        let bg = Block::default().style(Style::default().bg(t.popup_bg));
        f.render_widget(bg, size);

        let area = centered_rect(40, 5, size);
        f.render_widget(Clear, area);

        let block = Block::default()
            .title(" 󰦝 ProtonVPN ")
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .border_style(Style::default().fg(t.border_active))
            .style(Style::default().bg(t.popup_bg));

        let inner = block.inner(area);
        f.render_widget(block, area);

        let text = Paragraph::new(vec![
            Line::from(Span::styled(message, Style::default().fg(t.warning))),
            Line::from(Span::styled(
                "Press Esc to cancel",
                Style::default().fg(t.fg_muted),
            )),
        ])
        .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(text, inner);
    })?;
    Ok(())
}

/// Show error and wait for key press, returns true if user wants to retry
pub fn show_error<B: Backend>(terminal: &mut Terminal<B>, error: &str) -> Result<bool> {
    let mut form = LoginForm::new();
    let theme = load_theme();
    form.set_status(error, true);

    loop {
        terminal.draw(|f| {
            render_login(f, &form, &theme);

            // Override hints to show retry option
            let size = f.size();
            let login_area = centered_rect(50, 14, size);
            let inner = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(1),
                    Constraint::Min(0),
                ])
                .split(login_area);

            let hint_style = Style::default().fg(theme.info);
            let hint_bold = Style::default().fg(theme.info).add_modifier(Modifier::BOLD);
            let sep_style = Style::default().fg(theme.fg_muted);

            let hints = Line::from(vec![
                Span::styled("Enter", hint_bold),
                Span::styled(" Retry ", hint_style),
                Span::styled("| ", sep_style),
                Span::styled("Esc", hint_bold),
                Span::styled(" Quit", hint_style),
            ]);
            let hints_para = Paragraph::new(hints).alignment(ratatui::layout::Alignment::Center);
            f.render_widget(hints_para, inner[5]);
        })?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Enter => return Ok(true),
                    KeyCode::Esc => return Ok(false),
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        return Ok(false)
                    }
                    _ => {}
                }
            }
        }
    }
}
