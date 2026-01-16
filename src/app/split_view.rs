use super::{App, DisplayItem, SplitFocus};
use crate::config::AppConfig;

impl App {
    pub fn toggle_split_view(&mut self) {
        self.split_view = !self.split_view;

        if let Ok(mut config) = AppConfig::load() {
            config.split_view = self.split_view;
            let _ = config.save();
        }

        if self.split_view {
            // Entering split view - apply current search state
            if !self.search_query.is_empty() {
                self.update_split_view_for_search();
            } else {
                // Restore full country list (in case it was filtered before)
                self.country_list = self.full_country_list.clone();

                // Sync country selection with current tree view selection
                if let Some(idx) = self.state.selected() {
                    if let Some(item) = self.displayed_items.get(idx) {
                        let country_code = match item {
                            DisplayItem::CountryHeader(c) => c.clone(),
                            DisplayItem::EntryIpHeader(c, _) => c.clone(),
                            DisplayItem::Server(server_idx) => {
                                self.all_servers[*server_idx].exit_country.clone()
                            }
                        };
                        if let Some(pos) = self.country_list.iter().position(|c| c == &country_code)
                        {
                            self.country_state.select(Some(pos));
                        }
                    }
                }
                // Ensure we have a valid selection
                if self.country_state.selected().is_none() && !self.country_list.is_empty() {
                    self.country_state.select(Some(0));
                }
                self.update_server_list_for_selected_country();
            }
            self.split_focus = SplitFocus::Countries;
        } else {
            // Exiting split view - refresh tree view with current search state
            self.update_display_list();
            if !self.displayed_items.is_empty() && self.state.selected().is_none() {
                self.state.select(Some(0));
            }
        }
    }

    fn get_entry_ip_for_idx(&self, idx: usize) -> &str {
        &self.search_cache[idx].entry_ip
    }

    pub fn update_server_list_for_selected_country(&mut self) {
        self.split_server_items.clear();
        if let Some(idx) = self.country_state.selected() {
            if let Some(country_code) = self.country_list.get(idx) {
                // Collect server indices for this country
                let mut server_indices: Vec<usize> = self
                    .all_servers
                    .iter()
                    .enumerate()
                    .filter(|(_, s)| &s.exit_country == country_code)
                    .map(|(i, _)| i)
                    .collect();

                // Sort by entry IP then name for consistent grouping (using cached entry_ip)
                server_indices.sort_by(|&a, &b| {
                    self.search_cache[a]
                        .entry_ip
                        .cmp(&self.search_cache[b].entry_ip)
                        .then(self.all_servers[a].name.cmp(&self.all_servers[b].name))
                });

                if self.group_by_entry_ip {
                    let mut current_entry_ip = String::new();
                    for i in server_indices {
                        let entry_ip = self.get_entry_ip_for_idx(i);
                        if entry_ip != current_entry_ip {
                            current_entry_ip = entry_ip.to_string();
                            self.split_server_items.push(DisplayItem::EntryIpHeader(
                                country_code.clone(),
                                current_entry_ip.clone(),
                            ));
                        }
                        self.split_server_items.push(DisplayItem::Server(i));
                    }
                } else {
                    for i in server_indices {
                        self.split_server_items.push(DisplayItem::Server(i));
                    }
                }

                // Select first server (skip headers)
                self.select_first_server_in_split();
            }
        }
    }

    fn select_first_server_in_split(&mut self) {
        for (i, item) in self.split_server_items.iter().enumerate() {
            if matches!(item, DisplayItem::Server(_)) {
                self.server_state.select(Some(i));
                return;
            }
        }
        self.server_state.select(None);
    }

    pub fn update_split_view_for_search(&mut self) {
        if self.search_query.is_empty() {
            // Restore full country list and normal behavior
            self.country_list = self.full_country_list.clone();
            if !self.country_list.is_empty() {
                if self.country_state.selected().is_none() {
                    self.country_state.select(Some(0));
                }
                self.update_server_list_for_selected_country();
            }
            return;
        }

        let query = &self.search_query;

        // 1. Find ALL matching servers with scores (regardless of country)
        let mut scored_servers = self.collect_scored_servers(query);

        // Sort by score (lower is better), then by entry IP, then by server name
        // Using cached entry_ip to avoid repeated lookups
        scored_servers.sort_by(|a, b| {
            let score_cmp = a.1.cmp(&b.1);
            if score_cmp != std::cmp::Ordering::Equal {
                return score_cmp;
            }
            self.search_cache[a.0]
                .entry_ip
                .cmp(&self.search_cache[b.0].entry_ip)
                .then(self.all_servers[a.0].name.cmp(&self.all_servers[b.0].name))
        });

        // Build split_server_items with optional IP grouping
        self.split_server_items.clear();
        if self.group_by_entry_ip {
            let mut current_entry_ip = String::new();
            for (idx, _) in scored_servers {
                let entry_ip = self.get_entry_ip_for_idx(idx);
                let country = self.all_servers[idx].exit_country.clone();
                if entry_ip != current_entry_ip {
                    current_entry_ip = entry_ip.to_string();
                    self.split_server_items.push(DisplayItem::EntryIpHeader(
                        country,
                        current_entry_ip.clone(),
                    ));
                }
                self.split_server_items.push(DisplayItem::Server(idx));
            }
        } else {
            for (idx, _) in scored_servers {
                self.split_server_items.push(DisplayItem::Server(idx));
            }
        }

        // 2. Filter and sort country list by best match score
        // Uses cached country names to avoid repeated lookups
        let mut scored_countries = self.collect_scored_countries(query);

        // Build a map from code to cached name for sorting
        let country_name_map: std::collections::HashMap<&str, &str> = self
            .country_search_cache
            .iter()
            .map(|c| (c.code.as_str(), c.name.as_str()))
            .collect();

        scored_countries.sort_by(|a, b| {
            let score_cmp = a.1.cmp(&b.1);
            if score_cmp != std::cmp::Ordering::Equal {
                return score_cmp;
            }
            let name_a = country_name_map
                .get(a.0.as_str())
                .copied()
                .unwrap_or(a.0.as_str());
            let name_b = country_name_map
                .get(b.0.as_str())
                .copied()
                .unwrap_or(b.0.as_str());
            name_a.cmp(name_b)
        });

        self.country_list = scored_countries.into_iter().map(|(code, _)| code).collect();

        // 3. Reset selections
        self.select_first_server_in_split();
        if !self.country_list.is_empty() {
            self.country_state.select(Some(0));
        } else {
            self.country_state.select(None);
        }
    }

    fn find_next_server_index(&self, from: usize, forward: bool) -> Option<usize> {
        let len = self.split_server_items.len();
        if len == 0 {
            return None;
        }

        let mut idx = from;
        for _ in 0..len {
            idx = if forward {
                if idx >= len - 1 {
                    0
                } else {
                    idx + 1
                }
            } else if idx == 0 {
                len - 1
            } else {
                idx - 1
            };

            if matches!(
                self.split_server_items.get(idx),
                Some(DisplayItem::Server(_))
            ) {
                return Some(idx);
            }
        }
        None
    }

    pub fn split_next(&mut self) {
        match self.split_focus {
            SplitFocus::Countries => {
                let len = self.country_list.len();
                if len == 0 {
                    return;
                }
                let i = match self.country_state.selected() {
                    Some(i) => {
                        if i >= len - 1 {
                            0
                        } else {
                            i + 1
                        }
                    }
                    None => 0,
                };
                self.country_state.select(Some(i));
                self.update_server_list_for_selected_country();
            }
            SplitFocus::Servers => {
                if let Some(current) = self.server_state.selected() {
                    if let Some(next) = self.find_next_server_index(current, true) {
                        self.server_state.select(Some(next));
                    }
                }
            }
        }
    }

    pub fn split_previous(&mut self) {
        match self.split_focus {
            SplitFocus::Countries => {
                let len = self.country_list.len();
                if len == 0 {
                    return;
                }
                let i = match self.country_state.selected() {
                    Some(i) => {
                        if i == 0 {
                            len - 1
                        } else {
                            i - 1
                        }
                    }
                    None => 0,
                };
                self.country_state.select(Some(i));
                self.update_server_list_for_selected_country();
            }
            SplitFocus::Servers => {
                if let Some(current) = self.server_state.selected() {
                    if let Some(prev) = self.find_next_server_index(current, false) {
                        self.server_state.select(Some(prev));
                    }
                }
            }
        }
    }

    pub fn split_page_down(&mut self) {
        match self.split_focus {
            SplitFocus::Countries => {
                let len = self.country_list.len();
                if len == 0 {
                    return;
                }
                let i = match self.country_state.selected() {
                    Some(i) => std::cmp::min(i + 10, len - 1),
                    None => 0,
                };
                self.country_state.select(Some(i));
                self.update_server_list_for_selected_country();
            }
            SplitFocus::Servers => {
                let len = self.split_server_items.len();
                if len == 0 {
                    return;
                }
                let target = match self.server_state.selected() {
                    Some(i) => std::cmp::min(i + 10, len - 1),
                    None => 0,
                };
                // Find nearest server at or after target
                for i in target..len {
                    if matches!(self.split_server_items.get(i), Some(DisplayItem::Server(_))) {
                        self.server_state.select(Some(i));
                        return;
                    }
                }
                // If none found, select last server
                for i in (0..target).rev() {
                    if matches!(self.split_server_items.get(i), Some(DisplayItem::Server(_))) {
                        self.server_state.select(Some(i));
                        return;
                    }
                }
            }
        }
    }

    pub fn split_page_up(&mut self) {
        match self.split_focus {
            SplitFocus::Countries => {
                let len = self.country_list.len();
                if len == 0 {
                    return;
                }
                let i = match self.country_state.selected() {
                    Some(i) => i.saturating_sub(10),
                    None => 0,
                };
                self.country_state.select(Some(i));
                self.update_server_list_for_selected_country();
            }
            SplitFocus::Servers => {
                let len = self.split_server_items.len();
                if len == 0 {
                    return;
                }
                let target = self
                    .server_state
                    .selected()
                    .map(|i| i.saturating_sub(10))
                    .unwrap_or(0);
                // Find nearest server at or before target
                for i in (0..=target).rev() {
                    if matches!(self.split_server_items.get(i), Some(DisplayItem::Server(_))) {
                        self.server_state.select(Some(i));
                        return;
                    }
                }
                // If none found, select first server
                self.select_first_server_in_split();
            }
        }
    }

    pub fn split_go_to_first(&mut self) {
        match self.split_focus {
            SplitFocus::Countries => {
                if !self.country_list.is_empty() {
                    self.country_state.select(Some(0));
                    self.update_server_list_for_selected_country();
                }
            }
            SplitFocus::Servers => {
                self.select_first_server_in_split();
            }
        }
    }

    pub fn split_go_to_last(&mut self) {
        match self.split_focus {
            SplitFocus::Countries => {
                if !self.country_list.is_empty() {
                    self.country_state.select(Some(self.country_list.len() - 1));
                    self.update_server_list_for_selected_country();
                }
            }
            SplitFocus::Servers => {
                // Find last server
                for i in (0..self.split_server_items.len()).rev() {
                    if matches!(self.split_server_items.get(i), Some(DisplayItem::Server(_))) {
                        self.server_state.select(Some(i));
                        return;
                    }
                }
            }
        }
    }

    pub fn split_switch_focus(&mut self) {
        self.split_focus = match self.split_focus {
            SplitFocus::Countries => SplitFocus::Servers,
            SplitFocus::Servers => SplitFocus::Countries,
        };
    }

    pub fn get_selected_server_idx_in_split(&self) -> Option<usize> {
        if let Some(idx) = self.server_state.selected() {
            if let Some(DisplayItem::Server(server_idx)) = self.split_server_items.get(idx) {
                return Some(*server_idx);
            }
        }
        None
    }
}
