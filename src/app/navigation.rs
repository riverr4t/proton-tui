use super::{App, DisplayItem};

const PAGE_JUMP: usize = 10;

impl App {
    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.displayed_items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.displayed_items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn page_down(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                let next = i + PAGE_JUMP;
                if next >= self.displayed_items.len() {
                    self.displayed_items.len() - 1
                } else {
                    next
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn page_up(&mut self) {
        let i = match self.state.selected() {
            Some(i) => i.saturating_sub(PAGE_JUMP),
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn go_to_first(&mut self) {
        if !self.displayed_items.is_empty() {
            self.state.select(Some(0));
        }
    }

    pub fn go_to_last(&mut self) {
        if !self.displayed_items.is_empty() {
            self.state.select(Some(self.displayed_items.len() - 1));
        }
    }

    pub fn toggle_current_selection(&mut self) {
        if let Some(idx) = self.state.selected() {
            if let Some(item) = self.displayed_items.get(idx).cloned() {
                match item {
                    DisplayItem::CountryHeader(country) => {
                        if self.expanded_countries.contains(&country) {
                            self.expanded_countries.remove(&country);
                        } else {
                            self.expanded_countries.insert(country);
                        }
                        self.update_display_list();
                    }
                    DisplayItem::EntryIpHeader(country, entry_ip) => {
                        let key = (country, entry_ip);
                        if self.expanded_entry_ips.contains(&key) {
                            self.expanded_entry_ips.remove(&key);
                        } else {
                            self.expanded_entry_ips.insert(key);
                        }
                        self.update_display_list();
                    }
                    DisplayItem::RegionHeader(country, region) => {
                        let key = (country, region);
                        if self.expanded_regions.contains(&key) {
                            self.expanded_regions.remove(&key);
                        } else {
                            self.expanded_regions.insert(key);
                        }
                        self.update_display_list();
                    }
                    DisplayItem::Server(_) => {
                        // handled by connect_to_selected usually
                    }
                }
            }
        }
    }

    pub fn expand_selected(&mut self) {
        if let Some(idx) = self.state.selected() {
            if let Some(item) = self.displayed_items.get(idx).cloned() {
                match item {
                    DisplayItem::CountryHeader(country) => {
                        if !self.expanded_countries.contains(&country) {
                            self.expanded_countries.insert(country);
                            self.update_display_list();
                        }
                    }
                    DisplayItem::EntryIpHeader(country, entry_ip) => {
                        let key = (country, entry_ip);
                        if !self.expanded_entry_ips.contains(&key) {
                            self.expanded_entry_ips.insert(key);
                            self.update_display_list();
                        }
                    }
                    DisplayItem::RegionHeader(country, region) => {
                        let key = (country, region);
                        if !self.expanded_regions.contains(&key) {
                            self.expanded_regions.insert(key);
                            self.update_display_list();
                        }
                    }
                    DisplayItem::Server(_) => {}
                }
            }
        }
    }

    pub fn collapse_selected(&mut self) {
        if let Some(idx) = self.state.selected() {
            if let Some(item) = self.displayed_items.get(idx).cloned() {
                match item {
                    DisplayItem::CountryHeader(country) => {
                        if self.expanded_countries.contains(&country) {
                            self.expanded_countries.remove(&country);
                            self.update_display_list();
                        }
                    }
                    DisplayItem::EntryIpHeader(country, entry_ip) => {
                        let key = (country.clone(), entry_ip);
                        if self.expanded_entry_ips.contains(&key) {
                            self.expanded_entry_ips.remove(&key);
                            self.update_display_list();
                        } else {
                            // Already collapsed, collapse parent country
                            if self.expanded_countries.contains(&country) {
                                self.expanded_countries.remove(&country);
                                self.update_display_list();

                                // Find the country header and select it
                                if let Some(header_pos) =
                                    self.displayed_items.iter().position(|it| {
                                        matches!(it, DisplayItem::CountryHeader(c) if c == &country)
                                    })
                                {
                                    self.state.select(Some(header_pos));
                                }
                            }
                        }
                    }
                    DisplayItem::RegionHeader(country, region) => {
                        let key = (country.clone(), region.clone());
                        if self.expanded_regions.contains(&key) {
                            self.expanded_regions.remove(&key);
                            self.update_display_list();
                        } else {
                            // Already collapsed, collapse parent country
                            if self.expanded_countries.contains(&country) {
                                self.expanded_countries.remove(&country);
                                self.update_display_list();

                                if let Some(header_pos) =
                                    self.displayed_items.iter().position(|it| {
                                        matches!(it, DisplayItem::CountryHeader(c) if c == &country)
                                    })
                                {
                                    self.state.select(Some(header_pos));
                                }
                            }
                        }
                    }
                    DisplayItem::Server(server_idx) => {
                        let server = &self.all_servers[server_idx];
                        let country = server.exit_country.clone();

                        // Try collapsing region first
                        if let Some(rc) = self.search_cache[server_idx].region_code.clone() {
                            let region_key = (country.clone(), rc.clone());
                            if self.expanded_regions.contains(&region_key) {
                                self.expanded_regions.remove(&region_key);
                                self.update_display_list();

                                if let Some(header_pos) =
                                    self.displayed_items.iter().position(|it| {
                                        matches!(it, DisplayItem::RegionHeader(c, r) if c == &country && r == &rc)
                                    })
                                {
                                    self.state.select(Some(header_pos));
                                }
                                return;
                            }
                        }

                        // Fall back to collapsing entry IP group
                        let entry_ip = server
                            .servers
                            .first()
                            .map(|s| s.entry_ip.clone())
                            .unwrap_or_default();
                        let key = (country.clone(), entry_ip.clone());

                        if self.expanded_entry_ips.contains(&key) {
                            self.expanded_entry_ips.remove(&key);
                            self.update_display_list();

                            if let Some(header_pos) = self.displayed_items.iter().position(|it| {
                                matches!(it, DisplayItem::EntryIpHeader(c, ip) if c == &country && ip == &entry_ip)
                            }) {
                                self.state.select(Some(header_pos));
                            }
                        }
                    }
                }
            }
        }
    }
}
