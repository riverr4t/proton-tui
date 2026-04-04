use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use std::{io, time::Duration};

mod api;
mod app;
mod auth;
mod config;
mod countries;
mod login;
mod models;
mod regions;
mod theme;
mod tokens;
mod ui;
mod wireguard;

use api::ProtonClient;
use app::filter::{FEATURE_P2P, FEATURE_SC, FEATURE_STR, FEATURE_TOR};
use app::{App, ConfigTarget, FocusPanel, SplitFocus};
use auth::ProtonAuth;
use login::{run_login, show_authenticating, show_error, show_loading, LoginResult};
use tokens::{load_tokens, save_tokens, StoredTokens};

const POLL_INTERVAL_MS: u64 = 50;
const CANCEL_POLL_INTERVAL_MS: u64 = 50;

/// Authenticate using TUI login form
async fn authenticate_tui<B: Backend>(terminal: &mut Terminal<B>) -> Result<Option<StoredTokens>> {
    loop {
        // Show login form
        let result = run_login(terminal)?;

        match result {
            LoginResult::Cancel => return Ok(None),
            LoginResult::Submit { username, password } => {
                // Show authenticating status
                show_authenticating(terminal)?;

                // Perform authentication
                let auth = ProtonAuth::new()?;
                let auth_result = tokio::select! {
                    result = auth.authenticate(&username, &password) => Some(result),
                    _ = wait_for_cancel_key() => None,
                };

                match auth_result {
                    Some(Ok(auth_result)) => {
                        let tokens = StoredTokens::from(auth_result);
                        save_tokens(&tokens)?;
                        return Ok(Some(tokens));
                    }
                    Some(Err(e)) => {
                        let error_msg = e.to_string();
                        let short_error = if error_msg.len() > 40 {
                            format!("{}...", &error_msg[..40])
                        } else {
                            error_msg
                        };

                        if !show_error(terminal, &short_error)? {
                            return Ok(None);
                        }
                        // Loop back to login form for retry
                    }
                    None => return Ok(None),
                }
            }
        }
    }
}

/// Get tokens - either from disk or by authenticating via TUI
async fn get_tokens<B: Backend>(terminal: &mut Terminal<B>) -> Result<Option<StoredTokens>> {
    // Try to load saved tokens first
    if let Some(tokens) = load_tokens()? {
        return Ok(Some(tokens));
    }

    // No saved tokens, need to authenticate
    authenticate_tui(terminal).await
}

async fn wait_for_cancel_key() -> io::Result<()> {
    loop {
        let maybe_key = tokio::task::spawn_blocking(|| -> io::Result<Option<KeyEvent>> {
            if event::poll(Duration::from_millis(CANCEL_POLL_INTERVAL_MS))? {
                if let Event::Key(key) = event::read()? {
                    return Ok(Some(key));
                }
            }
            Ok(None)
        })
        .await
        .map_err(io::Error::other)??;

        if let Some(key) = maybe_key {
            if key.code == KeyCode::Esc
                || (key.code == KeyCode::Char('c')
                    && key.modifiers.contains(event::KeyModifiers::CONTROL))
            {
                return Ok(());
            }
        }
    }
}

fn cleanup_terminal<B: Backend + io::Write>(terminal: &mut Terminal<B>) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

enum LoopAction {
    Continue,
    Exit,
}

async fn handle_normal_mode_key(app: &mut App, key: KeyEvent) -> io::Result<LoopAction> {
    // Handle favorites panel navigation when focused
    if app.focus_panel == FocusPanel::Favorites {
        let fav_count = app.get_favorite_servers().len();
        match key.code {
            KeyCode::Down | KeyCode::Char('j') => {
                if fav_count > 0 {
                    let i = app.favorites_state.selected().unwrap_or(0);
                    app.favorites_state
                        .select(Some(if i >= fav_count - 1 { 0 } else { i + 1 }));
                }
                return Ok(LoopAction::Continue);
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if fav_count > 0 {
                    let i = app.favorites_state.selected().unwrap_or(0);
                    app.favorites_state
                        .select(Some(if i == 0 { fav_count - 1 } else { i - 1 }));
                }
                return Ok(LoopAction::Continue);
            }
            KeyCode::Enter => {
                // Connect to selected favorite
                if let Some(selected) = app.favorites_state.selected() {
                    let fav_servers = app.get_favorite_servers();
                    if let Some(&(server_idx, _)) = fav_servers.get(selected) {
                        if app.connection_status.is_some() {
                            app.stop_wireguard().await;
                        }
                        if let Some(config_path) =
                            app.create_config(server_idx, ConfigTarget::Runtime).await
                        {
                            if let Some(server) = app.all_servers.get(server_idx) {
                                let name = server.name.clone();
                                app.start_wireguard(
                                    config_path.to_str().unwrap_or("wg0.conf"),
                                    name,
                                )
                                .await;
                            }
                        }
                    }
                }
                return Ok(LoopAction::Continue);
            }
            KeyCode::Char('F') => {
                // Unfavorite from favorites panel
                if let Some(selected) = app.favorites_state.selected() {
                    let fav_servers = app.get_favorite_servers();
                    if let Some(&(_, server)) = fav_servers.get(selected) {
                        let id = server.id.clone();
                        drop(fav_servers);
                        app.toggle_favorite(&id);
                        // Adjust selection if needed
                        let new_count = app.get_favorite_servers().len();
                        if new_count == 0 {
                            app.focus_panel = FocusPanel::Main;
                            app.favorites_state.select(None);
                        } else if selected >= new_count {
                            app.favorites_state.select(Some(new_count - 1));
                        }
                    }
                }
                return Ok(LoopAction::Continue);
            }
            KeyCode::Esc => {
                app.focus_panel = FocusPanel::Main;
                return Ok(LoopAction::Continue);
            }
            // Let Tab, BackTab, and other global keys fall through
            KeyCode::Tab | KeyCode::BackTab => {}
            KeyCode::Char('q') => return Ok(LoopAction::Exit),
            KeyCode::Char('?') => {
                app.show_help_popup = true;
                return Ok(LoopAction::Continue);
            }
            _ => return Ok(LoopAction::Continue),
        }
    }

    match key.code {
        KeyCode::Char('q') => return Ok(LoopAction::Exit),
        KeyCode::Char('?') => {
            app.show_help_popup = true;
        }
        KeyCode::Char('v') => {
            app.toggle_split_view();
        }
        KeyCode::Char('i') => {
            app.toggle_group_by_entry_ip();
        }
        KeyCode::Char('f') => {
            app.show_filter_popup = true;
            app.filter_popup_selected = 0;
        }
        KeyCode::Char('F') => {
            // Toggle favorite on selected server
            let server_id = if app.split_view {
                app.get_selected_server_id_in_split()
            } else {
                app.get_selected_server_id()
            };
            if let Some(id) = server_id {
                app.toggle_favorite(&id);
            }
        }
        KeyCode::Char('c') => {
            // Quick-toggle Secure Core only filter
            app.toggle_feature_filter(FEATURE_SC);
        }
        KeyCode::Char('A') => {
            // Toggle auto-connect on selected server
            let server_id = if app.split_view {
                app.get_selected_server_id_in_split()
            } else {
                app.get_selected_server_id()
            };
            if let Some(id) = server_id {
                if app.auto_connect_id.as_deref() == Some(&id) {
                    app.set_auto_connect(None);
                } else {
                    app.set_auto_connect(Some(id));
                }
            }
        }
        KeyCode::Tab => {
            let has_favorites = !app.favorites.is_empty();
            if app.focus_panel == FocusPanel::Favorites {
                // Move from favorites to main panel
                app.focus_panel = FocusPanel::Main;
            } else if has_favorites && !app.split_view {
                // Move from main panel to favorites (tree view)
                app.focus_panel = FocusPanel::Favorites;
                if app.favorites_state.selected().is_none() {
                    app.favorites_state.select(Some(0));
                }
            } else if app.split_view {
                if has_favorites {
                    // Cycle: Countries → Servers → Favorites → Countries
                    match app.split_focus {
                        SplitFocus::Countries => {
                            app.split_focus = SplitFocus::Servers;
                        }
                        SplitFocus::Servers => {
                            app.focus_panel = FocusPanel::Favorites;
                            if app.favorites_state.selected().is_none() {
                                app.favorites_state.select(Some(0));
                            }
                        }
                    }
                } else {
                    app.split_switch_focus();
                }
            }
        }
        KeyCode::BackTab => {
            let has_favorites = !app.favorites.is_empty();
            if app.focus_panel == FocusPanel::Favorites {
                if app.split_view {
                    app.focus_panel = FocusPanel::Main;
                    app.split_focus = SplitFocus::Servers;
                } else {
                    app.focus_panel = FocusPanel::Main;
                }
            } else if app.split_view {
                if app.split_focus == SplitFocus::Countries && has_favorites {
                    app.focus_panel = FocusPanel::Favorites;
                    if app.favorites_state.selected().is_none() {
                        app.favorites_state.select(Some(0));
                    }
                } else {
                    app.split_switch_focus();
                }
            } else if has_favorites {
                app.focus_panel = FocusPanel::Favorites;
                if app.favorites_state.selected().is_none() {
                    app.favorites_state.select(Some(0));
                }
            }
        }
        KeyCode::Char('s') => {
            if app.split_view {
                if let Some(server_idx) = app.get_selected_server_idx_in_split() {
                    let _ = app.create_config(server_idx, ConfigTarget::Saved).await;
                }
            } else {
                app.save_selected_config().await;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.split_view {
                app.split_next();
            } else {
                app.next();
            }
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if app.split_view {
                app.split_previous();
            } else {
                app.previous();
            }
        }
        KeyCode::Left | KeyCode::Char('h') => {
            if app.split_view {
                if app.split_focus == SplitFocus::Servers {
                    app.split_focus = SplitFocus::Countries;
                }
            } else {
                app.collapse_selected();
            }
        }
        KeyCode::Right | KeyCode::Char('l') => {
            if app.split_view {
                if app.split_focus == SplitFocus::Countries {
                    app.split_focus = SplitFocus::Servers;
                }
            } else {
                app.expand_selected();
            }
        }
        KeyCode::PageDown => {
            if app.split_view {
                app.split_page_down();
            } else {
                app.page_down();
            }
        }
        KeyCode::Char('d') if !key.modifiers.contains(event::KeyModifiers::CONTROL) => {
            if app.connection_status.is_some() {
                app.stop_wireguard().await;
            }
        }
        KeyCode::Char('d') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
            if app.split_view {
                app.split_page_down();
            } else {
                app.page_down();
            }
        }
        KeyCode::PageUp => {
            if app.split_view {
                app.split_page_up();
            } else {
                app.page_up();
            }
        }
        KeyCode::Char('u') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
            if app.split_view {
                app.split_page_up();
            } else {
                app.page_up();
            }
        }
        KeyCode::Home | KeyCode::Char('g') => {
            if app.split_view {
                app.split_go_to_first();
            } else {
                app.go_to_first();
            }
        }
        KeyCode::End | KeyCode::Char('G') => {
            if app.split_view {
                app.split_go_to_last();
            } else {
                app.go_to_last();
            }
        }
        KeyCode::Enter => {
            if app.split_view {
                if app.split_focus == SplitFocus::Servers {
                    if let Some(server_idx) = app.get_selected_server_idx_in_split() {
                        if app.connection_status.is_some() {
                            app.stop_wireguard().await;
                        }
                        if let Some(config_path) =
                            app.create_config(server_idx, ConfigTarget::Runtime).await
                        {
                            if let Some(server) = app.all_servers.get(server_idx) {
                                app.start_wireguard(
                                    config_path.to_str().unwrap_or("wg0.conf"),
                                    server.name.clone(),
                                )
                                .await;
                            }
                        }
                    }
                } else {
                    // Switch to servers pane when Enter is pressed on country
                    app.split_focus = SplitFocus::Servers;
                }
            } else {
                app.connect_to_selected().await;
            }
        }
        KeyCode::Char(' ') => {
            if !app.split_view {
                app.toggle_current_selection();
            }
        }
        KeyCode::Esc => {
            if app.split_view {
                app.toggle_split_view();
            }
        }
        _ => {}
    }

    Ok(LoopAction::Continue)
}

#[tokio::main]
async fn main() -> Result<()> {
    // TUI Setup (early, for login screen)
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Get authentication tokens
    let tokens = match get_tokens(&mut terminal).await? {
        Some(t) => t,
        None => {
            // User cancelled - cleanup and exit
            cleanup_terminal(&mut terminal)?;
            return Ok(());
        }
    };

    let client = ProtonClient::new(tokens.uid, tokens.access_token);

    // Prevent Ctrl+C from killing the app (useful when running sudo subprocess)
    tokio::spawn(async move {
        loop {
            let _ = tokio::signal::ctrl_c().await;
        }
    });

    // Show loading screen while fetching servers
    show_loading(&mut terminal, "Loading servers...")?;

    let servers_result = tokio::select! {
        result = client.get_logical_servers() => Some(result),
        _ = wait_for_cancel_key() => None,
    };

    let mut servers = match servers_result {
        Some(Ok(s)) => s,
        Some(Err(e)) => {
            // If we get an auth error, delete saved tokens
            let err_str = e.to_string();
            if err_str.contains("401")
                || err_str.contains("Unauthorized")
                || err_str.contains("Invalid")
            {
                tokens::delete_tokens()?;
                // Show error and offer to retry
                if show_error(&mut terminal, "Session expired. Press Enter to login again")? {
                    // Restart auth flow
                    match authenticate_tui(&mut terminal).await? {
                        Some(new_tokens) => {
                            let new_client =
                                ProtonClient::new(new_tokens.uid, new_tokens.access_token);
                            new_client.get_logical_servers().await?
                        }
                        None => {
                            cleanup_terminal(&mut terminal)?;
                            return Ok(());
                        }
                    }
                } else {
                    cleanup_terminal(&mut terminal)?;
                    return Ok(());
                }
            } else {
                cleanup_terminal(&mut terminal)?;
                return Err(e);
            }
        }
        None => {
            cleanup_terminal(&mut terminal)?;
            return Ok(());
        }
    };

    // Sort by country
    servers.sort_by(|a, b| {
        let name_a = countries::get_country_name(&a.exit_country);
        let name_b = countries::get_country_name(&b.exit_country);
        name_a.cmp(&name_b).then(a.name.cmp(&b.name))
    });

    let mut app = App::new(client, servers);

    // Detect if a previous VPN connection is still active
    app.detect_existing_connection();

    // Auto-connect if configured (skip if already connected)
    if app.connection_status.is_none() {
        if let Some(ref auto_id) = app.auto_connect_id.clone() {
            if let Some(server_idx) = app.all_servers.iter().position(|s| s.id == *auto_id) {
                app.log(format!(
                    "Auto-connecting to {}...",
                    app.all_servers[server_idx].name
                ));
                if let Some(config_path) =
                    app.create_config(server_idx, ConfigTarget::Runtime).await
                {
                    let server_name = app.all_servers[server_idx].name.clone();
                    app.start_wireguard(config_path.to_str().unwrap_or("wg0.conf"), server_name)
                        .await;
                }
            } else {
                app.log(format!("Auto-connect server '{}' not found", auto_id));
            }
        }
    }

    let res = run_app(&mut terminal, app).await;

    // TUI Cleanup
    cleanup_terminal(&mut terminal)?;

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        if app.should_redraw {
            terminal.clear()?;
            app.should_redraw = false;
        }

        // Update stats if connected (polling every frame)
        if app.connection_status.is_some() {
            app.update_traffic_stats();
        }

        terminal.draw(|f| ui::ui(f, &mut app))?;

        if event::poll(std::time::Duration::from_millis(POLL_INTERVAL_MS))? {
            if let Event::Key(key) = event::read()? {
                // Global Ctrl+C handler
                if key.code == KeyCode::Char('c')
                    && key.modifiers.contains(event::KeyModifiers::CONTROL)
                {
                    continue;
                }

                // Help Popup Handling (any key closes it)
                if app.show_help_popup {
                    app.show_help_popup = false;
                    continue;
                }

                // Filter Popup Handling
                if app.show_filter_popup {
                    handle_filter_popup_key(&mut app, key);
                    continue;
                }

                let action = handle_normal_mode_key(&mut app, key).await?;

                if let LoopAction::Exit = action {
                    return Ok(());
                }
            }
        }
    }
}

const FILTER_POPUP_ITEMS: usize = 9; // 0-8

fn handle_filter_popup_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc | KeyCode::Char('f') => {
            app.show_filter_popup = false;
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.filter_popup_selected = (app.filter_popup_selected + 1) % FILTER_POPUP_ITEMS;
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.filter_popup_selected = if app.filter_popup_selected == 0 {
                FILTER_POPUP_ITEMS - 1
            } else {
                app.filter_popup_selected - 1
            };
        }
        KeyCode::Enter | KeyCode::Char(' ') => match app.filter_popup_selected {
            0 => app.cycle_load_filter(),
            1 => app.toggle_feature_filter(FEATURE_SC),
            2 => app.toggle_feature_filter(FEATURE_TOR),
            3 => app.toggle_feature_filter(FEATURE_P2P),
            4 => app.toggle_feature_filter(FEATURE_STR),
            5 => app.toggle_online_filter(),
            6 => {
                app.active_filter.favorites_only = !app.active_filter.favorites_only;
                app.refresh_after_filter();
            }
            7 => {
                // Toggle sort: first press cycles field, space cycles direction
                if key.code == KeyCode::Char(' ') {
                    app.toggle_sort_direction();
                } else {
                    app.toggle_sort_field();
                }
            }
            8 => app.reset_filters(),
            _ => {}
        },
        _ => {}
    }
}
