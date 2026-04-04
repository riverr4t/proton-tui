mod connection;
pub mod filter;
mod navigation;
mod search;
mod split_view;
mod state;

pub use connection::ConfigTarget;
pub use search::ServerSearchCache;
pub use state::{
    ConnectionStatus, DisplayItem, FocusPanel, ServerFilter, SortDirection, SortField, SplitFocus,
};

use ratatui::widgets::ListState;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::time::Instant;

use crate::api::ProtonClient;
use crate::config::AppConfig;
use crate::countries;
use crate::models::LogicalServer;
use crate::theme::Theme;

pub struct App {
    pub all_servers: Vec<LogicalServer>,
    pub sorted_server_indices: Vec<usize>, // Pre-sorted by country, entry IP, name
    pub displayed_items: Vec<DisplayItem>,
    pub state: ListState,
    pub client: ProtonClient,
    pub total_servers: usize,
    pub unique_entry_ips: usize,
    pub status_message: String,
    pub expanded_countries: HashSet<String>,
    pub expanded_entry_ips: HashSet<(String, String)>, // (country_code, entry_ip)
    pub expanded_regions: HashSet<(String, String)>,   // (country_code, region_code)
    pub group_by_entry_ip: bool,
    pub server_counts: HashMap<String, usize>,
    pub should_redraw: bool,
    pub connection_status: Option<ConnectionStatus>,
    pub show_help_popup: bool,
    pub current_config_id: Option<String>,
    // Split view state
    pub split_view: bool,
    pub split_focus: SplitFocus,
    pub country_list: Vec<String>,
    pub full_country_list: Vec<String>,
    pub country_state: ListState,
    pub split_server_items: Vec<DisplayItem>,
    pub server_state: ListState,
    // Pre-computed cache (avoids repeated to_lowercase calls)
    pub search_cache: Vec<ServerSearchCache>,
    // Theme
    pub theme: Theme,
    // Filter & sort
    pub active_filter: ServerFilter,
    pub sort_field: SortField,
    pub sort_direction: SortDirection,
    pub show_filter_popup: bool,
    pub filter_popup_selected: usize,
    // Favorites
    pub favorites: HashSet<String>,
    pub auto_connect_id: Option<String>,
    pub favorites_state: ListState,
    pub focus_panel: FocusPanel,
}

impl App {
    pub fn new(client: ProtonClient, servers: Vec<LogicalServer>) -> App {
        let mut counts = HashMap::new();
        for server in &servers {
            *counts.entry(server.exit_country.clone()).or_insert(0) += 1;
        }

        // Build sorted country list
        let mut country_list: Vec<String> = counts.keys().cloned().collect();
        country_list.sort_by_key(|a| countries::get_country_name(a));

        // Build cache (pre-compute lowercase strings once)
        let search_cache = Self::build_search_cache(&servers);

        // Pre-compute sorted server indices using cached data
        let mut sorted_indices: Vec<usize> = (0..servers.len()).collect();
        sorted_indices.sort_by(|&a, &b| {
            let cache_a = &search_cache[a];
            let cache_b = &search_cache[b];
            let country_cmp = cache_a.country_name.cmp(&cache_b.country_name);
            if country_cmp != std::cmp::Ordering::Equal {
                return country_cmp;
            }
            // Region name: None sorts last
            let region_cmp = match (&cache_a.region_name, &cache_b.region_name) {
                (Some(a_r), Some(b_r)) => a_r.cmp(b_r),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => std::cmp::Ordering::Equal,
            };
            if region_cmp != std::cmp::Ordering::Equal {
                return region_cmp;
            }
            let entry_ip_cmp = cache_a.entry_ip.cmp(&cache_b.entry_ip);
            if entry_ip_cmp != std::cmp::Ordering::Equal {
                return entry_ip_cmp;
            }
            servers[a].name.cmp(&servers[b].name)
        });

        // Compute unique entry IPs
        let mut entry_ips: HashSet<&str> = HashSet::new();
        for server in &servers {
            for instance in &server.servers {
                entry_ips.insert(&instance.entry_ip);
            }
        }
        let total_servers = servers.len();
        let unique_entry_ips = entry_ips.len();

        let config = AppConfig::load().unwrap_or_default();
        let group_by_entry_ip = config.group_by_entry_ip;
        let split_view = config.split_view;
        let theme = Theme::from_name(&config.theme);
        let favorites: HashSet<String> = config.favorites.into_iter().collect();
        let auto_connect_id = config.auto_connect;

        let mut app = App {
            all_servers: servers,
            sorted_server_indices: sorted_indices,
            displayed_items: Vec::new(),
            state: ListState::default(),
            client,
            total_servers,
            unique_entry_ips,
            status_message: String::new(),
            expanded_countries: HashSet::new(),
            expanded_entry_ips: HashSet::new(),
            expanded_regions: HashSet::new(),
            group_by_entry_ip,
            server_counts: counts,
            should_redraw: false,
            connection_status: None,
            show_help_popup: false,
            current_config_id: None,
            // Split view state
            split_view,
            split_focus: SplitFocus::Countries,
            full_country_list: country_list.clone(),
            country_list,
            country_state: ListState::default(),
            split_server_items: Vec::new(),
            server_state: ListState::default(),
            // Pre-computed cache
            search_cache,
            // Theme
            theme,
            // Filter & sort
            active_filter: ServerFilter::default(),
            sort_field: SortField::Country,
            sort_direction: SortDirection::Ascending,
            show_filter_popup: false,
            filter_popup_selected: 0,
            // Favorites
            favorites,
            auto_connect_id,
            favorites_state: ListState::default(),
            focus_panel: FocusPanel::Main,
        };
        app.update_display_list();
        if !app.displayed_items.is_empty() {
            app.state.select(Some(0));
        }
        // Initialize country selection for split view
        if !app.country_list.is_empty() {
            app.country_state.select(Some(0));
            app.update_server_list_for_selected_country();
        }
        app
    }

    pub fn log(&mut self, msg: String) {
        self.status_message = msg;
    }

    pub fn toggle_group_by_entry_ip(&mut self) {
        self.group_by_entry_ip = !self.group_by_entry_ip;
        self.expanded_regions.clear();
        self.expanded_entry_ips.clear();
        if self.split_view {
            self.update_server_list_for_selected_country();
        } else {
            self.update_display_list();
        }
    }

    pub fn speed_to_human(bytes_per_sec: f64) -> String {
        if bytes_per_sec < 1024.0 {
            return format!("{:.0} B/s", bytes_per_sec);
        }
        let kb = bytes_per_sec / 1024.0;
        if kb < 1024.0 {
            return format!("{:.1} KB/s", kb);
        }
        let mb = kb / 1024.0;
        if mb < 1024.0 {
            return format!("{:.1} MB/s", mb);
        }
        let gb = mb / 1024.0;
        format!("{:.1} GB/s", gb)
    }

    pub fn bytes_to_human(b: u64) -> String {
        const UNIT: u64 = 1024;
        if b < UNIT {
            return format!("{} B", b);
        }
        let div = UNIT;
        if b < div * UNIT {
            return format!("{:.1} KB", b as f64 / div as f64);
        }
        let div = div * UNIT;
        if b < div * UNIT {
            return format!("{:.1} MB", b as f64 / div as f64);
        }
        let div = div * UNIT;
        format!("{:.1} GB", b as f64 / div as f64)
    }

    pub fn update_traffic_stats(&mut self) {
        if let Some(ref mut status) = self.connection_status {
            if let Ok(content) = fs::read_to_string("/proc/net/dev") {
                for line in content.lines() {
                    let parts: Vec<&str> = line.split(':').collect();
                    if parts.len() == 2 {
                        let iface = parts[0].trim();
                        if iface == status.interface {
                            let stats: Vec<&str> = parts[1].split_whitespace().collect();
                            if stats.len() >= 9 {
                                if let (Ok(rx), Ok(tx)) =
                                    (stats[0].parse::<u64>(), stats[8].parse::<u64>())
                                {
                                    status.rx_bytes = rx;
                                    status.tx_bytes = tx;

                                    // Compute speed every ~500ms
                                    let now = Instant::now();
                                    let elapsed = now.duration_since(status.last_sample_time);
                                    if elapsed.as_millis() >= 500 {
                                        let dt = elapsed.as_secs_f64();
                                        if status.prev_rx_bytes > 0 {
                                            let rx_delta =
                                                rx.saturating_sub(status.prev_rx_bytes) as f64;
                                            let tx_delta =
                                                tx.saturating_sub(status.prev_tx_bytes) as f64;
                                            status.rx_speed = rx_delta / dt;
                                            status.tx_speed = tx_delta / dt;

                                            // Push to history ring buffer (max 60 samples)
                                            if status.rx_history.len() >= 60 {
                                                status.rx_history.remove(0);
                                            }
                                            if status.tx_history.len() >= 60 {
                                                status.tx_history.remove(0);
                                            }
                                            status.rx_history.push(status.rx_speed as u64);
                                            status.tx_history.push(status.tx_speed as u64);
                                        }
                                        status.prev_rx_bytes = rx;
                                        status.prev_tx_bytes = tx;
                                        status.last_sample_time = now;
                                    }
                                }
                            }
                            break;
                        }
                    }
                }
            }
        }
    }

    pub fn is_favorite(&self, server_id: &str) -> bool {
        self.favorites.contains(server_id)
    }

    pub fn toggle_favorite(&mut self, server_id: &str) {
        if self.favorites.contains(server_id) {
            self.favorites.remove(server_id);
            self.log(format!("Removed from favorites: {}", server_id));
        } else {
            self.favorites.insert(server_id.to_string());
            self.log(format!("Added to favorites: {}", server_id));
        }
        self.save_favorites();
    }

    pub fn set_auto_connect(&mut self, server_id: Option<String>) {
        if let Some(ref id) = server_id {
            self.log(format!("Auto-connect set to: {}", id));
        } else {
            self.log("Auto-connect cleared".to_string());
        }
        self.auto_connect_id = server_id;
        self.save_favorites();
    }

    fn save_favorites(&self) {
        if let Ok(mut config) = AppConfig::load() {
            config.favorites = self.favorites.iter().cloned().collect();
            config.auto_connect = self.auto_connect_id.clone();
            let _ = config.save();
        }
    }

    pub fn get_selected_server_id(&self) -> Option<String> {
        let idx = self.state.selected()?;
        let item = self.displayed_items.get(idx)?;
        if let DisplayItem::Server(server_idx) = item {
            Some(self.all_servers[*server_idx].id.clone())
        } else {
            None
        }
    }

    pub fn get_favorite_servers(&self) -> Vec<(usize, &LogicalServer)> {
        self.all_servers
            .iter()
            .enumerate()
            .filter(|(_, s)| self.favorites.contains(&s.id))
            .collect()
    }

    pub fn get_selected_server_id_in_split(&self) -> Option<String> {
        let idx = self.server_state.selected()?;
        if let Some(DisplayItem::Server(server_idx)) = self.split_server_items.get(idx) {
            Some(self.all_servers[*server_idx].id.clone())
        } else {
            None
        }
    }
}
