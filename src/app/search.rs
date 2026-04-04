use super::{App, DisplayItem};
use crate::countries;
use crate::models::LogicalServer;
use crate::regions;

/// Pre-computed data for a server.
/// Avoids repeated lookups during sorting and display.
pub struct ServerSearchCache {
    pub country_name: String, // For sort comparisons
    pub entry_ip: String,
    pub region_code: Option<String>,
    pub region_name: Option<String>,
    pub city_with_state: String,
}

impl App {
    /// Build cache for all servers (called once at startup)
    pub fn build_search_cache(servers: &[LogicalServer]) -> Vec<ServerSearchCache> {
        servers
            .iter()
            .map(|server| {
                let country_name = countries::get_country_name(&server.exit_country);
                let entry_ip = server
                    .servers
                    .first()
                    .map(|s| s.entry_ip.clone())
                    .unwrap_or_default();
                let region =
                    regions::resolve_region(&server.exit_country, &server.name, &server.city);
                let (region_code, region_name) = match &region {
                    Some((code, name)) => (Some(code.clone()), Some(name.clone())),
                    None => (None, None),
                };
                let city_with_state = match &region_code {
                    Some(code) if !server.city.is_empty() => {
                        format!("{}, {}", server.city, code)
                    }
                    _ => server.city.clone(),
                };
                ServerSearchCache {
                    country_name,
                    entry_ip,
                    region_code,
                    region_name,
                    city_with_state,
                }
            })
            .collect()
    }

    pub(crate) fn passes_filter(&self, server_idx: usize) -> bool {
        let server = &self.all_servers[server_idx];
        if !self.active_filter.matches(server) {
            return false;
        }
        if self.active_filter.favorites_only && !self.favorites.contains(&server.id) {
            return false;
        }
        true
    }

    pub fn update_display_list(&mut self) {
        self.displayed_items.clear();

        // Use pre-sorted indices for fast expand/collapse
        let mut current_country = String::new();
        let mut current_region = String::new();
        let mut current_entry_ip = String::new();
        let filter_active = self.active_filter.is_active();

        for &i in &self.sorted_server_indices {
            if filter_active && !self.passes_filter(i) {
                continue;
            }

            let exit_country = &self.all_servers[i].exit_country;
            let cache = &self.search_cache[i];
            let entry_ip = &cache.entry_ip;

            if *exit_country != current_country {
                current_country = exit_country.clone();
                current_region.clear();
                current_entry_ip.clear();
                self.displayed_items
                    .push(DisplayItem::CountryHeader(current_country.clone()));
            }

            if self.expanded_countries.contains(&current_country) {
                if self.group_by_entry_ip {
                    if let Some(ref rc) = cache.region_code {
                        // US server with region: use RegionHeader instead of EntryIpHeader
                        if *rc != current_region {
                            current_region = rc.clone();
                            self.displayed_items.push(DisplayItem::RegionHeader(
                                current_country.clone(),
                                current_region.clone(),
                            ));
                        }
                        let region_key = (current_country.clone(), current_region.clone());
                        if self.expanded_regions.contains(&region_key) {
                            self.displayed_items.push(DisplayItem::Server(i));
                        }
                    } else {
                        // Non-US: use existing EntryIpHeader logic
                        if *entry_ip != current_entry_ip {
                            current_entry_ip = entry_ip.clone();
                            self.displayed_items.push(DisplayItem::EntryIpHeader(
                                current_country.clone(),
                                current_entry_ip.clone(),
                            ));
                        }
                        let entry_ip_key = (current_country.clone(), current_entry_ip.clone());
                        if self.expanded_entry_ips.contains(&entry_ip_key) {
                            self.displayed_items.push(DisplayItem::Server(i));
                        }
                    }
                } else {
                    // Flat mode: show servers directly under country
                    self.displayed_items.push(DisplayItem::Server(i));
                }
            }
        }
    }
}
