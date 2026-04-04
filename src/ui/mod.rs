use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{block::BorderType, Block, Borders, Clear, List, ListItem, Paragraph, Sparkline},
    Frame,
};

use crate::app::filter::{FEATURE_P2P, FEATURE_SC, FEATURE_STR, FEATURE_TOR};
use crate::app::{App, DisplayItem, FocusPanel, SplitFocus};
use crate::countries;
use crate::theme::Theme;

const SPLIT_COUNTRY_PERCENT: u16 = 35;
const SPLIT_SERVER_PERCENT: u16 = 65;

fn feature_badges<'a>(features: i32, theme: &Theme) -> Vec<Span<'a>> {
    let mut spans = Vec::new();
    if features & 1 != 0 {
        spans.push(Span::styled(
            " SC ",
            Style::default().fg(theme.popup_bg).bg(theme.secure_core),
        ));
        spans.push(Span::raw(" "));
    }
    if features & 2 != 0 {
        spans.push(Span::styled(
            " TOR ",
            Style::default().fg(theme.popup_bg).bg(theme.warning),
        ));
        spans.push(Span::raw(" "));
    }
    if features & 4 != 0 {
        spans.push(Span::styled(
            " P2P ",
            Style::default().fg(theme.popup_bg).bg(theme.success),
        ));
        spans.push(Span::raw(" "));
    }
    if features & 8 != 0 {
        spans.push(Span::styled(
            " STR ",
            Style::default().fg(theme.popup_bg).bg(theme.info),
        ));
        spans.push(Span::raw(" "));
    }
    spans
}

fn load_bar<'a>(load: i32, color: ratatui::style::Color, compact: bool) -> Vec<Span<'a>> {
    if compact {
        return vec![Span::styled(
            format!("{:>3}%", load),
            Style::default().fg(color),
        )];
    }
    let width = 10;
    let filled = (load as usize * width / 100).min(width);
    let empty = width - filled;
    vec![
        Span::styled("█".repeat(filled), Style::default().fg(color)),
        Span::styled("░".repeat(empty), Style::default().fg(color)),
        Span::styled(format!(" {:>3}%", load), Style::default().fg(color)),
    ]
}

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
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

pub fn ui(frame: &mut Frame, app: &mut App) {
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Min(0)])
        .split(frame.size());

    let fav_servers = app.get_favorite_servers();
    let has_favorites = !fav_servers.is_empty();
    let fav_count = fav_servers.len();
    drop(fav_servers);

    let fav_height = if has_favorites {
        (fav_count as u16 + 2).min(7) // +2 for borders, cap at 7
    } else {
        0
    };

    let status_height = if app.connection_status.is_some() {
        14
    } else {
        4
    };

    let chunks = if has_favorites {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(fav_height),    // Favorites panel
                Constraint::Length(0),             // Search bar (hidden)
                Constraint::Min(0),                // List
                Constraint::Length(status_height), // Status
                Constraint::Length(1),             // Keybinding hints
            ])
            .split(outer[0])
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(0),             // No favorites
                Constraint::Length(0),             // Search bar (hidden)
                Constraint::Min(0),                // List
                Constraint::Length(status_height), // Status
                Constraint::Length(1),             // Keybinding hints
            ])
            .split(outer[0])
    };

    // --- Favorites Panel ---
    if has_favorites {
        render_favorites_panel(frame, app, chunks[0]);
    }

    // --- List (Tree View or Split View) ---
    if app.split_view {
        render_split_view(frame, app, chunks[2]);
    } else {
        render_tree_view(frame, app, chunks[2]);
    }

    // --- Footer (Status) ---
    render_status_bar(frame, app, chunks[3]);

    // --- Keybinding Hints Bar ---
    render_hints_bar(frame, app, chunks[4]);

    // --- Popups ---
    if app.show_help_popup {
        render_help_popup(frame, &app.theme);
    }

    if app.show_filter_popup {
        render_filter_popup(frame, app);
    }
}

fn render_favorites_panel(frame: &mut Frame, app: &mut App, area: Rect) {
    let t = &app.theme;
    let focused = app.focus_panel == FocusPanel::Favorites;
    let border_type = if focused {
        BorderType::Thick
    } else {
        BorderType::default()
    };
    let border_style = if focused {
        Style::default().fg(t.border_active)
    } else {
        Style::default()
    };
    let title_style = if focused {
        Style::default().fg(t.accent).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };

    let fav_servers = app.get_favorite_servers();
    let items: Vec<ListItem> = fav_servers
        .iter()
        .map(|(_, s)| {
            let load_color = if s.load < 30 {
                t.load_low
            } else if s.load < 70 {
                t.load_medium
            } else {
                t.load_high
            };
            let country_name = countries::get_country_name(&s.exit_country);
            let country_flag = countries::get_country_flag(&s.exit_country);

            let mut spans = vec![
                Span::styled("󰓎 ", Style::default().fg(t.accent)),
                Span::styled(format!("{:<12}", s.name), Style::default().fg(t.fg)),
                Span::styled(
                    format!(" {} {} ", country_flag, country_name),
                    Style::default().fg(t.fg_dim),
                ),
                Span::styled(format!("{:>3}%", s.load), Style::default().fg(load_color)),
            ];
            spans.push(Span::raw("  "));
            spans.extend(feature_badges(s.features, t));

            ListItem::new(Line::from(spans))
        })
        .collect();

    let highlight_style = if focused {
        Style::default()
            .bg(t.highlight_bg)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().bg(t.highlight_inactive_bg)
    };

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(border_type)
                .title(" Favorites ")
                .title_style(title_style)
                .border_style(border_style),
        )
        .highlight_style(highlight_style)
        .highlight_symbol(if focused { "► " } else { "  " });

    frame.render_stateful_widget(list, area, &mut app.favorites_state);
}

fn render_tree_view(frame: &mut Frame, app: &mut App, area: Rect) {
    let t = &app.theme;
    let servers_title = add_connected_badge("ProtonVPN Servers", app.connection_status.is_some());

    let items: Vec<ListItem> = app
        .displayed_items
        .iter()
        .map(|item| match item {
            DisplayItem::CountryHeader(country_code) => {
                let country_full_name = countries::get_country_name(country_code);
                let country_flag = countries::get_country_flag(country_code);
                let count = app.server_counts.get(country_code).unwrap_or(&0);
                let is_expanded = app.expanded_countries.contains(country_code);

                let (icon, style) = if is_expanded {
                    (
                        "▼",
                        Style::default().fg(t.accent).add_modifier(Modifier::BOLD),
                    )
                } else {
                    ("▶", Style::default().fg(t.fg))
                };

                ListItem::new(Line::from(vec![
                    Span::styled(format!(" {} ", icon), style),
                    Span::styled(format!("{} ", country_flag), Style::default()),
                    Span::styled(format!("{} ", country_full_name), style),
                    Span::styled(
                        format!("({}) ", country_code),
                        style.add_modifier(Modifier::DIM),
                    ),
                    Span::styled(format!("[{}]", count), Style::default().fg(t.fg_muted)),
                ]))
            }
            DisplayItem::EntryIpHeader(country_code, entry_ip) => {
                let key = (country_code.clone(), entry_ip.clone());
                let is_expanded = app.expanded_entry_ips.contains(&key);

                let server_count = app
                    .all_servers
                    .iter()
                    .filter(|s| {
                        &s.exit_country == country_code
                            && s.servers.first().map(|i| &i.entry_ip) == Some(entry_ip)
                    })
                    .count();

                let (icon, style) = if is_expanded {
                    (
                        "▼",
                        Style::default().fg(t.success).add_modifier(Modifier::BOLD),
                    )
                } else {
                    ("▶", Style::default().fg(t.fg_muted))
                };

                ListItem::new(Line::from(vec![
                    Span::styled(format!("   {} ", icon), style),
                    Span::styled(format!("{} ", entry_ip), style),
                    Span::styled(
                        format!("[{}]", server_count),
                        Style::default().fg(t.fg_muted),
                    ),
                ]))
            }
            DisplayItem::RegionHeader(country_code, region_code) => {
                let region_name = crate::regions::get_region_name(country_code, region_code)
                    .unwrap_or(region_code.as_str());
                let key = (country_code.clone(), region_code.clone());
                let is_expanded = app.expanded_regions.contains(&key);

                let server_count = app
                    .all_servers
                    .iter()
                    .enumerate()
                    .filter(|(i, s)| {
                        s.exit_country == *country_code
                            && app.search_cache[*i].region_code.as_deref()
                                == Some(region_code.as_str())
                    })
                    .count();

                let (icon, style) = if is_expanded {
                    (
                        "▼",
                        Style::default().fg(t.info).add_modifier(Modifier::BOLD),
                    )
                } else {
                    ("▶", Style::default().fg(t.fg_muted))
                };

                ListItem::new(Line::from(vec![
                    Span::styled(format!("   {} ", icon), style),
                    Span::styled(format!("{} ({}) ", region_name, region_code), style),
                    Span::styled(
                        format!("[{}]", server_count),
                        Style::default().fg(t.fg_muted),
                    ),
                ]))
            }
            DisplayItem::Server(idx) => {
                let s = &app.all_servers[*idx];
                let load_color = if s.load < 30 {
                    t.load_low
                } else if s.load < 70 {
                    t.load_medium
                } else {
                    t.load_high
                };

                let compact = frame.size().width < 100;
                let fav = if app.is_favorite(&s.id) {
                    "󰓎 "
                } else {
                    "  "
                };
                let connector = if s.is_secure_core() {
                    "      ╰═ "
                } else {
                    "      ╰─ "
                };

                let mut spans = vec![
                    Span::styled(
                        connector,
                        Style::default().fg(if s.is_secure_core() {
                            t.secure_core
                        } else {
                            t.fg_muted
                        }),
                    ),
                    Span::styled(fav, Style::default().fg(t.accent)),
                    Span::styled(format!("{:<10}", s.name), Style::default().fg(t.fg)),
                ];
                // Show SC route: "Entry → Exit"
                if s.is_secure_core() && s.entry_country != s.exit_country {
                    let entry_name = countries::get_country_name(&s.entry_country);
                    let exit_name = countries::get_country_name(&s.exit_country);
                    spans.push(Span::styled(
                        format!(" {} → {} ", entry_name, exit_name),
                        Style::default().fg(t.secure_core),
                    ));
                } else {
                    let city_display = &app.search_cache[*idx].city_with_state;
                    spans.push(Span::styled(
                        format!(" {:<15} ", city_display),
                        Style::default().fg(t.fg_dim),
                    ));
                }
                spans.push(Span::styled(" ", Style::default()));
                spans.extend(load_bar(s.load, load_color, compact));
                spans.push(Span::raw("  "));
                if !compact {
                    spans.extend(feature_badges(s.features, t));
                }

                ListItem::new(Line::from(spans))
            }
        })
        .collect();

    let tree_focused = app.focus_panel == FocusPanel::Main;
    let tree_border_type = if tree_focused {
        BorderType::Thick
    } else {
        BorderType::default()
    };
    let tree_border_style = if tree_focused {
        Style::default().fg(t.border_active)
    } else {
        Style::default()
    };
    let tree_title_style = if tree_focused {
        Style::default().fg(t.accent).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };

    let servers_list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(tree_border_type)
                .title(servers_title)
                .title_style(tree_title_style)
                .border_style(tree_border_style),
        )
        .highlight_style(
            Style::default()
                .bg(t.highlight_bg)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(" ");

    frame.render_stateful_widget(servers_list, area, &mut app.state);
}

fn render_split_view(frame: &mut Frame, app: &mut App, area: Rect) {
    let t = &app.theme;
    let split_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(SPLIT_COUNTRY_PERCENT),
                Constraint::Percentage(SPLIT_SERVER_PERCENT),
            ]
            .as_ref(),
        )
        .split(area);

    // Country list
    let country_focused = app.split_focus == SplitFocus::Countries;
    let country_border_type = if country_focused {
        BorderType::Thick
    } else {
        BorderType::default()
    };
    let country_border_style = if country_focused {
        Style::default().fg(t.border_active)
    } else {
        Style::default()
    };
    let country_title_style = if country_focused {
        Style::default().fg(t.accent).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };

    let country_title = format!(" Countries ({}) ", app.full_country_list.len());

    {
        let country_items: Vec<ListItem> = app
            .country_list
            .iter()
            .map(|country_code| {
                let country_name = countries::get_country_name(country_code);
                let country_flag = countries::get_country_flag(country_code);
                let count = app.server_counts.get(country_code).unwrap_or(&0);

                ListItem::new(Line::from(vec![
                    Span::styled(format!(" {} ", country_flag), Style::default()),
                    Span::styled(format!("{} ", country_name), Style::default().fg(t.fg)),
                    Span::styled(format!("[{}]", count), Style::default().fg(t.fg_muted)),
                ]))
            })
            .collect();

        let (country_highlight_style, country_highlight_symbol) =
            if app.split_focus == SplitFocus::Countries {
                (
                    Style::default()
                        .bg(t.highlight_bg)
                        .add_modifier(Modifier::BOLD),
                    "► ",
                )
            } else {
                (Style::default().bg(t.highlight_inactive_bg), "  ")
            };

        let country_list = List::new(country_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(country_border_type)
                    .title(country_title)
                    .title_style(country_title_style)
                    .border_style(country_border_style),
            )
            .highlight_style(country_highlight_style)
            .highlight_symbol(country_highlight_symbol);

        frame.render_stateful_widget(country_list, split_chunks[0], &mut app.country_state);
    }

    // Server list for selected country
    let server_focused = app.split_focus == SplitFocus::Servers;
    let server_border_type = if server_focused {
        BorderType::Thick
    } else {
        BorderType::default()
    };
    let server_border_style = if server_focused {
        Style::default().fg(t.border_active)
    } else {
        Style::default()
    };
    let server_title_style = if server_focused {
        Style::default().fg(t.accent).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };

    let server_count = app
        .split_server_items
        .iter()
        .filter(|item| matches!(item, DisplayItem::Server(_)))
        .count();
    let selected_country_name = app
        .country_state
        .selected()
        .and_then(|idx| app.country_list.get(idx))
        .map(|code| {
            format!(
                "Servers - {} ({}/{})",
                countries::get_country_name(code),
                server_count,
                app.all_servers.len()
            )
        })
        .unwrap_or_else(|| format!("Servers ({})", app.all_servers.len()));

    let server_list_title =
        add_connected_badge(&selected_country_name, app.connection_status.is_some());

    {
        let server_items: Vec<ListItem> = app
            .split_server_items
            .iter()
            .map(|item| match item {
                DisplayItem::EntryIpHeader(_, entry_ip) => {
                    ListItem::new(Line::from(vec![Span::styled(
                        format!(" {} ", entry_ip),
                        Style::default().fg(t.success).add_modifier(Modifier::BOLD),
                    )]))
                }
                DisplayItem::Server(idx) => {
                    let s = &app.all_servers[*idx];
                    let load_color = if s.load < 30 {
                        t.load_low
                    } else if s.load < 70 {
                        t.load_medium
                    } else {
                        t.load_high
                    };

                    let compact = frame.size().width < 100;
                    let fav = if app.is_favorite(&s.id) {
                        "󰓎 "
                    } else {
                        "  "
                    };
                    let mut spans = vec![
                        Span::styled(fav, Style::default().fg(t.accent)),
                        Span::styled(format!("{:<12}", s.name), Style::default().fg(t.fg)),
                    ];
                    if s.is_secure_core() && s.entry_country != s.exit_country {
                        let entry_name = countries::get_country_name(&s.entry_country);
                        let exit_name = countries::get_country_name(&s.exit_country);
                        spans.push(Span::styled(
                            format!("{} → {} ", entry_name, exit_name),
                            Style::default().fg(t.secure_core),
                        ));
                    } else {
                        let city_display = &app.search_cache[*idx].city_with_state;
                        spans.push(Span::styled(
                            format!("{:<15}", city_display),
                            Style::default().fg(t.fg_dim),
                        ));
                    }
                    spans.extend(load_bar(s.load, load_color, compact));
                    spans.push(Span::raw("  "));
                    if !compact {
                        spans.extend(feature_badges(s.features, t));
                    }

                    ListItem::new(Line::from(spans))
                }
                DisplayItem::RegionHeader(country_code, region_code) => {
                    let region_name = crate::regions::get_region_name(country_code, region_code)
                        .unwrap_or(region_code.as_str());
                    ListItem::new(Line::from(vec![Span::styled(
                        format!(" {} ({}) ", region_name, region_code),
                        Style::default().fg(t.info).add_modifier(Modifier::BOLD),
                    )]))
                }
                DisplayItem::CountryHeader(_) => ListItem::new(Line::from("")),
            })
            .collect();

        let (server_highlight_style, server_highlight_symbol) =
            if app.split_focus == SplitFocus::Servers {
                (
                    Style::default()
                        .bg(t.highlight_bg)
                        .add_modifier(Modifier::BOLD),
                    "► ",
                )
            } else {
                (Style::default().bg(t.highlight_inactive_bg), "  ")
            };

        let server_list = List::new(server_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(server_border_type)
                    .title(server_list_title)
                    .title_style(server_title_style)
                    .border_style(server_border_style),
            )
            .highlight_style(server_highlight_style)
            .highlight_symbol(server_highlight_symbol);

        frame.render_stateful_widget(server_list, split_chunks[1], &mut app.server_state);
    }
}

fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let t = &app.theme;

    if let Some(status) = &app.connection_status {
        // Expanded connected view with sparklines
        let duration = status.connected_at.elapsed();
        let secs = duration.as_secs();
        let h = secs / 3600;
        let m = (secs % 3600) / 60;
        let s = secs % 60;

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::default())
            .title(" Status ")
            .border_style(Style::default().fg(t.border_active));
        let inner_area = block.inner(area);
        frame.render_widget(block, area);

        let sub_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Info text (status/server/speeds)
                Constraint::Length(3), // RX sparkline with border
                Constraint::Length(3), // TX sparkline with border
                Constraint::Length(1), // Disconnect hint
            ])
            .split(inner_area);

        // Info lines
        let line1 = Line::from(vec![
            Span::styled(
                " 󰌘 ",
                Style::default().fg(t.success).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Connected",
                Style::default().fg(t.success).add_modifier(Modifier::BOLD),
            ),
            Span::styled("  ", Style::default()),
            Span::styled(&status.server_name, Style::default().fg(t.fg)),
            Span::styled("  ", Style::default()),
            Span::styled(&status.interface, Style::default().fg(t.fg_dim)),
            Span::styled("  ", Style::default()),
            Span::styled(
                format!("{:02}:{:02}:{:02}", h, m, s),
                Style::default().fg(t.fg),
            ),
        ]);
        let line2 = Line::from(vec![
            Span::styled(
                format!(" ↓ {} ", App::speed_to_human(status.rx_speed)),
                Style::default().fg(t.info).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("({})", App::bytes_to_human(status.rx_bytes)),
                Style::default().fg(t.fg_dim),
            ),
            Span::styled("  ", Style::default()),
            Span::styled(
                format!("↑ {} ", App::speed_to_human(status.tx_speed)),
                Style::default().fg(t.upload).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("({})", App::bytes_to_human(status.tx_bytes)),
                Style::default().fg(t.fg_dim),
            ),
        ]);
        let info = Paragraph::new(vec![line1, line2]);
        frame.render_widget(info, sub_layout[0]);

        // RX sparkline
        let rx_sparkline = Sparkline::default()
            .block(
                Block::default()
                    .title(format!(
                        " 󰇚 Download: {} ",
                        App::speed_to_human(status.rx_speed)
                    ))
                    .borders(Borders::ALL)
                    .border_type(BorderType::default())
                    .border_style(Style::default().fg(t.border)),
            )
            .data(&status.rx_history)
            .style(Style::default().fg(t.info));
        frame.render_widget(rx_sparkline, sub_layout[1]);

        // TX sparkline
        let tx_sparkline = Sparkline::default()
            .block(
                Block::default()
                    .title(format!(
                        " 󰕒 Upload: {} ",
                        App::speed_to_human(status.tx_speed)
                    ))
                    .borders(Borders::ALL)
                    .border_type(BorderType::default())
                    .border_style(Style::default().fg(t.border)),
            )
            .data(&status.tx_history)
            .style(Style::default().fg(t.upload));
        frame.render_widget(tx_sparkline, sub_layout[2]);

        // Disconnect hint
        let hint = Paragraph::new(Line::from(Span::styled(
            "Press 'd' to disconnect",
            Style::default().fg(t.error).add_modifier(Modifier::BOLD),
        )))
        .alignment(Alignment::Center);
        frame.render_widget(hint, sub_layout[3]);
    } else {
        // Disconnected compact view
        let filter_info = if app.active_filter.is_active() {
            format!(" | Filtered: {}", app.active_filter.active_count())
        } else {
            String::new()
        };
        let fav_count = app.favorites.len();
        let fav_info = if fav_count > 0 {
            format!(" | Favorites: {}", fav_count)
        } else {
            String::new()
        };
        let sort_info = format!(
            " | Sort: {} {}",
            app.sort_field.label(),
            app.sort_direction.indicator()
        );

        let line1 = if !app.status_message.is_empty() {
            format!(" 󰌙 {}", app.status_message)
        } else {
            " 󰌙 Ready".to_string()
        };

        let line2 = format!(
            " Servers: {} | Entry IPs: {}{}{}{}",
            app.total_servers, app.unique_entry_ips, filter_info, fav_info, sort_info
        );

        let footer = Paragraph::new(vec![Line::from(line1), Line::from(line2)])
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::default())
                    .title(" Status ")
                    .border_style(Style::default()),
            )
            .style(Style::default().fg(t.accent));

        frame.render_widget(footer, area);
    }
}

fn render_hints_bar(frame: &mut Frame, app: &App, area: Rect) {
    let t = &app.theme;
    let key_style = Style::default().fg(t.info).add_modifier(Modifier::BOLD);
    let sep_style = Style::default().fg(t.fg_muted);
    let desc_style = Style::default().fg(t.info);

    let connected = app.connection_status.is_some();

    let hints = if app.show_help_popup {
        Line::from(vec![
            Span::styled("Any Key", key_style),
            Span::styled(" Close", desc_style),
        ])
    } else if app.focus_panel == FocusPanel::Favorites {
        Line::from(vec![
            Span::styled("Tab", key_style),
            Span::styled(" Switch ", desc_style),
            Span::styled("| ", sep_style),
            Span::styled("Enter", key_style),
            Span::styled(" Connect ", desc_style),
            Span::styled("| ", sep_style),
            Span::styled("F", key_style),
            Span::styled(" Unfav ", desc_style),
            Span::styled("| ", sep_style),
            Span::styled("?", key_style),
            Span::styled(" Help ", desc_style),
            Span::styled("| ", sep_style),
            Span::styled("q", key_style),
            Span::styled(" Quit", desc_style),
        ])
    } else if app.split_view {
        match app.split_focus {
            SplitFocus::Countries => Line::from(vec![
                Span::styled("Tab", key_style),
                Span::styled(" Switch ", desc_style),
                Span::styled("| ", sep_style),
                Span::styled("Enter", key_style),
                Span::styled(" Select ", desc_style),
                Span::styled("| ", sep_style),
                Span::styled("v", key_style),
                Span::styled(" Tree ", desc_style),
                Span::styled("| ", sep_style),
                Span::styled("?", key_style),
                Span::styled(" Help ", desc_style),
                Span::styled("| ", sep_style),
                Span::styled("q", key_style),
                Span::styled(" Quit", desc_style),
            ]),
            SplitFocus::Servers => {
                let mut spans = vec![
                    Span::styled("Tab", key_style),
                    Span::styled(" Switch ", desc_style),
                    Span::styled("| ", sep_style),
                    Span::styled("Enter", key_style),
                    Span::styled(" Connect ", desc_style),
                ];
                if connected {
                    spans.extend([
                        Span::styled("| ", sep_style),
                        Span::styled("d", key_style),
                        Span::styled(" Disconnect ", desc_style),
                    ]);
                }
                spans.extend([
                    Span::styled("| ", sep_style),
                    Span::styled("F", key_style),
                    Span::styled(" Fav ", desc_style),
                    Span::styled("| ", sep_style),
                    Span::styled("←", key_style),
                    Span::styled(" Back ", desc_style),
                    Span::styled("| ", sep_style),
                    Span::styled("?", key_style),
                    Span::styled(" Help ", desc_style),
                    Span::styled("| ", sep_style),
                    Span::styled("q", key_style),
                    Span::styled(" Quit", desc_style),
                ]);
                Line::from(spans)
            }
        }
    } else {
        let mut spans = vec![
            Span::styled("f", key_style),
            Span::styled(" Filter ", desc_style),
            Span::styled("| ", sep_style),
            Span::styled("F", key_style),
            Span::styled(" Fav ", desc_style),
            Span::styled("| ", sep_style),
            Span::styled("Enter", key_style),
            Span::styled(" Connect ", desc_style),
        ];
        if connected {
            spans.extend([
                Span::styled("| ", sep_style),
                Span::styled("d", key_style),
                Span::styled(" Disconnect ", desc_style),
            ]);
        }
        spans.extend([
            Span::styled("| ", sep_style),
            Span::styled("v", key_style),
            Span::styled(" Split ", desc_style),
            Span::styled("| ", sep_style),
            Span::styled("?", key_style),
            Span::styled(" Help ", desc_style),
            Span::styled("| ", sep_style),
            Span::styled("q", key_style),
            Span::styled(" Quit", desc_style),
        ]);
        Line::from(spans)
    };

    let hints_bar = Paragraph::new(hints).alignment(Alignment::Center);
    frame.render_widget(hints_bar, area);
}

fn add_connected_badge(title: &str, connected: bool) -> String {
    if connected {
        format!(" {} · Connected ", title)
    } else {
        format!(" {} ", title)
    }
}

fn render_help_popup(frame: &mut Frame, theme: &Theme) {
    let bg_color = theme.help_bg;
    let border_color = theme.help_border;
    let accent_color = theme.help_accent;
    let key_fg = theme.help_key_fg;
    let key_bg = theme.help_key_bg;
    let desc_color = theme.help_desc;
    let section_color = theme.help_section;
    let divider_color = theme.help_divider;
    let footer_bg = theme.help_footer_bg;
    let footer_text = theme.help_footer_fg;

    // Fixed column widths for table-like alignment
    const KEY_WIDTH: usize = 10;
    const DESC_WIDTH: usize = 18;
    const LEFT_PAD: usize = 5;
    const COL_GAP: usize = 2;

    // Calculate content width: pad + key + space + desc + gap + key + space + desc + pad
    let content_width =
        LEFT_PAD + KEY_WIDTH + 1 + DESC_WIDTH + COL_GAP + KEY_WIDTH + 1 + DESC_WIDTH + LEFT_PAD;
    // Add borders (2) and margin (2)
    let popup_width = (content_width + 4) as u16;

    let block = Block::default()
        .title(" Keyboard Shortcuts ")
        .title_style(
            Style::default()
                .fg(accent_color)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(bg_color));

    // Center the popup with fixed width
    let frame_size = frame.size();
    let popup_width = popup_width.min(frame_size.width);
    let popup_height = (frame_size.height * 85 / 100).min(frame_size.height);
    let x = (frame_size.width.saturating_sub(popup_width)) / 2;
    let y = (frame_size.height.saturating_sub(popup_height)) / 2;
    let area = Rect::new(x, y, popup_width, popup_height);

    frame.render_widget(Clear, area);
    frame.render_widget(block, area);

    let inner_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)].as_ref())
        .margin(1)
        .split(area);

    let content_area = inner_area[0];
    let footer_area = inner_area[1];

    // Styles
    let section_style = Style::default()
        .fg(section_color)
        .add_modifier(Modifier::BOLD);
    let key_style = Style::default()
        .fg(key_fg)
        .bg(key_bg)
        .add_modifier(Modifier::BOLD);
    let desc_style = Style::default().fg(desc_color);
    let divider_style = Style::default().fg(divider_color);

    let left_pad = " ".repeat(LEFT_PAD);
    let col_gap = " ".repeat(COL_GAP);

    // Helper to create a formatted key with background color, padded to fixed width
    let fmt_key = |key: &str| -> Span {
        Span::styled(
            format!(" {:^width$} ", key, width = KEY_WIDTH - 2),
            key_style,
        )
    };

    // Helper to create description padded to fixed width
    let fmt_desc = |desc: &str| -> Span {
        Span::styled(format!("{:<width$}", desc, width = DESC_WIDTH), desc_style)
    };

    // Helper to create a two-column row with consistent alignment
    let make_row = |key1: &str, desc1: &str, key2: &str, desc2: &str| -> Line {
        Line::from(vec![
            Span::raw(left_pad.clone()),
            fmt_key(key1),
            Span::raw(" "),
            fmt_desc(desc1),
            Span::raw(col_gap.clone()),
            fmt_key(key2),
            Span::raw(" "),
            fmt_desc(desc2),
        ])
    };

    // Divider width = content width minus left/right padding
    let divider_width = KEY_WIDTH + 1 + DESC_WIDTH + COL_GAP + KEY_WIDTH + 1 + DESC_WIDTH;
    let divider = "─".repeat(divider_width);

    let help_text = vec![
        // Navigation Section
        Line::from(vec![
            Span::raw(left_pad.clone()),
            Span::styled("  Navigation", section_style),
        ]),
        Line::from(""),
        make_row("↑ / k", "Move up", "↓ / j", "Move down"),
        make_row("Home / g", "Go to first", "End / G", "Go to last"),
        make_row("PgUp", "Page up", "PgDn", "Page down"),
        make_row("Ctrl-u", "Half page up", "Ctrl-d", "Half page down"),
        Line::from(""),
        Line::from(Span::styled(
            format!("{}{}", left_pad, divider),
            divider_style,
        )),
        Line::from(""),
        // Tree View Section
        Line::from(vec![
            Span::raw(left_pad.clone()),
            Span::styled("  Tree View", section_style),
        ]),
        Line::from(""),
        make_row("→ / l", "Expand node", "← / h", "Collapse node"),
        make_row("Space", "Toggle expand", "Enter", "Connect/Toggle"),
        Line::from(""),
        Line::from(Span::styled(
            format!("{}{}", left_pad, divider),
            divider_style,
        )),
        Line::from(""),
        // Split View Section
        Line::from(vec![
            Span::raw(left_pad.clone()),
            Span::styled("  Split View", section_style),
        ]),
        Line::from(""),
        make_row("v", "Toggle split", "Tab", "Switch pane"),
        make_row("← / →", "Switch pane", "Enter", "Connect server"),
        Line::from(""),
        Line::from(Span::styled(
            format!("{}{}", left_pad, divider),
            divider_style,
        )),
        Line::from(""),
        // Actions Section
        Line::from(vec![
            Span::raw(left_pad.clone()),
            Span::styled("  Actions", section_style),
        ]),
        Line::from(""),
        make_row("/", "Search servers", "s", "Save config"),
        make_row("i", "Toggle IP group", "f", "Filter & sort"),
        make_row("F", "Toggle favorite", "A", "Auto-connect"),
        make_row("c", "Secure Core", "d", "Disconnect VPN"),
        make_row("?", "Show this help", "q", "Quit application"),
    ];

    let paragraph = Paragraph::new(help_text);
    frame.render_widget(paragraph, content_area);

    // Footer with close instruction - centered with empty line before
    let footer = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            " Press any key to close ",
            Style::default().fg(footer_text).bg(footer_bg),
        )]),
    ])
    .alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(footer, footer_area);
}

fn render_filter_popup(frame: &mut Frame, app: &App) {
    let t = &app.theme;
    let area = centered_rect(50, 60, frame.size());

    let block = Block::default()
        .title(" Filter & Sort ")
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .border_style(Style::default().fg(t.border_active))
        .style(Style::default().bg(t.popup_bg));

    frame.render_widget(Clear, area);
    frame.render_widget(block, area);

    let inner_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(2)].as_ref())
        .margin(1)
        .split(area);

    let content_area = inner_area[0];
    let footer_area = inner_area[1];

    let selected = app.filter_popup_selected;
    let filter = &app.active_filter;

    let highlight = |idx: usize, text: &str| -> Style {
        if idx == selected {
            Style::default().fg(t.accent).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(text.parse::<i32>().map_or(t.fg, |_| t.fg))
        }
    };
    let _ = highlight; // suppress unused; we'll use a simpler approach

    let make_item = |idx: usize, label: &str, value: &str| -> Line {
        let is_sel = idx == selected;
        let prefix = if is_sel { "► " } else { "  " };
        let label_style = if is_sel {
            Style::default().fg(t.accent).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(t.fg_dim)
        };
        let value_style = if is_sel {
            Style::default().fg(t.fg).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(t.fg)
        };
        Line::from(vec![
            Span::styled(prefix, label_style),
            Span::styled(format!("{:<18}", label), label_style),
            Span::styled(value.to_string(), value_style),
        ])
    };

    // Row 0: Max load
    let load_val = match filter.max_load {
        Some(v) => format!("≤ {}%", v),
        None => "Any".to_string(),
    };

    // Row 1: Secure Core
    let sc_on = filter.features.is_some_and(|f| f & FEATURE_SC != 0);
    // Row 2: Tor
    let tor_on = filter.features.is_some_and(|f| f & FEATURE_TOR != 0);
    // Row 3: P2P
    let p2p_on = filter.features.is_some_and(|f| f & FEATURE_P2P != 0);
    // Row 4: Streaming
    let str_on = filter.features.is_some_and(|f| f & FEATURE_STR != 0);
    // Row 5: Online only
    let online_val = if filter.online_only { "Yes" } else { "No" };
    // Row 6: Favorites only
    let fav_val = if filter.favorites_only { "Yes" } else { "No" };
    // Row 7: Sort field
    let sort_val = format!(
        "{} {}",
        app.sort_field.label(),
        app.sort_direction.indicator()
    );
    // Row 8: Reset

    let check = |on: bool| -> &str {
        if on {
            "[x]"
        } else {
            "[ ]"
        }
    };

    let lines = vec![
        Line::from(""),
        make_item(0, "Max Load", &load_val),
        Line::from(""),
        Line::from(Span::styled(
            "  Feature Filters",
            Style::default().fg(t.fg_dim).add_modifier(Modifier::BOLD),
        )),
        make_item(1, "Secure Core", check(sc_on)),
        make_item(2, "Tor", check(tor_on)),
        make_item(3, "P2P", check(p2p_on)),
        make_item(4, "Streaming", check(str_on)),
        Line::from(""),
        make_item(5, "Online Only", online_val),
        make_item(6, "Favorites Only", fav_val),
        Line::from(""),
        Line::from(Span::styled(
            "  Sorting",
            Style::default().fg(t.fg_dim).add_modifier(Modifier::BOLD),
        )),
        make_item(7, "Sort By", &sort_val),
        Line::from(""),
        make_item(8, "Reset All", ""),
    ];

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, content_area);

    // Footer
    let fkey_style = Style::default().fg(t.info).add_modifier(Modifier::BOLD);
    let fdesc_style = Style::default().fg(t.info);
    let fsep_style = Style::default().fg(t.fg_muted);

    let footer_line = Line::from(vec![
        Span::styled("Enter", fkey_style),
        Span::styled(" Toggle ", fdesc_style),
        Span::styled("| ", fsep_style),
        Span::styled("↑↓", fkey_style),
        Span::styled(" Navigate ", fdesc_style),
        Span::styled("| ", fsep_style),
        Span::styled("Esc", fkey_style),
        Span::styled(" Close", fdesc_style),
    ]);
    let footer = Paragraph::new(vec![Line::from(""), footer_line])
        .alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(footer, footer_area);
}
