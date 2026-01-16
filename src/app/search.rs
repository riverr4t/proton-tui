use super::{App, DisplayItem};
use crate::countries;
use crate::models::LogicalServer;

/// Pre-computed lowercase search data for a server.
/// Avoids repeated `to_lowercase()` calls during search.
pub struct ServerSearchCache {
    pub name_lower: String,
    pub city_lower: String,
    pub country_code_lower: String,
    pub country_name_lower: String,
    pub country_name: String, // For sort comparisons
    pub entry_ip: String,
}

/// Pre-computed lowercase search data for a country.
pub struct CountrySearchCache {
    pub code: String,
    pub code_lower: String,
    pub name: String,
    pub name_lower: String,
}

impl App {
    /// Build search cache for all servers (called once at startup)
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
                ServerSearchCache {
                    name_lower: server.name.to_lowercase(),
                    city_lower: server.city.to_lowercase(),
                    country_code_lower: server.exit_country.to_lowercase(),
                    country_name_lower: country_name.to_lowercase(),
                    country_name,
                    entry_ip,
                }
            })
            .collect()
    }

    /// Build search cache for all countries (called once at startup)
    pub fn build_country_search_cache(country_codes: &[String]) -> Vec<CountrySearchCache> {
        country_codes
            .iter()
            .map(|code| {
                let name = countries::get_country_name(code);
                CountrySearchCache {
                    code: code.clone(),
                    code_lower: code.to_lowercase(),
                    name_lower: name.to_lowercase(),
                    name,
                }
            })
            .collect()
    }

    /// Fuzzy match on pre-lowercased text
    fn fuzzy_match_lower(text_lower: &str, query_lower: &str) -> bool {
        let mut text_chars = text_lower.chars();
        for q_char in query_lower.chars() {
            if !text_chars.any(|t_char| t_char == q_char) {
                return false;
            }
        }
        true
    }

    /// Match score using pre-lowercased text (avoids allocation)
    fn match_score_lower(text_lower: &str, query_lower: &str) -> Option<u8> {
        if query_lower.is_empty() {
            return Some(0);
        }

        // Score 0: Exact prefix match
        if text_lower.starts_with(query_lower) {
            return Some(0);
        }

        // Score 1: Word boundary prefix match
        for word in text_lower.split_whitespace() {
            if word.starts_with(query_lower) {
                return Some(1);
            }
        }
        for part in text_lower.split(['-', '_', '#']) {
            if part.starts_with(query_lower) {
                return Some(1);
            }
        }

        // Score 2: Contains match
        if text_lower.contains(query_lower) {
            return Some(2);
        }

        // Score 3: Fuzzy match
        if Self::fuzzy_match_lower(text_lower, query_lower) {
            return Some(3);
        }

        None
    }

    /// Score a server using pre-computed cache
    fn score_server_cached(cache: &ServerSearchCache, query_lower: &str) -> Option<u8> {
        let country_score = [
            Self::match_score_lower(&cache.country_name_lower, query_lower),
            Self::match_score_lower(&cache.country_code_lower, query_lower),
        ]
        .into_iter()
        .flatten()
        .min();
        let city_score = Self::match_score_lower(&cache.city_lower, query_lower);
        let name_score = Self::match_score_lower(&cache.name_lower, query_lower);

        [country_score, city_score, name_score]
            .into_iter()
            .flatten()
            .min()
    }

    pub(crate) fn collect_scored_servers(&self, query: &str) -> Vec<(usize, u8)> {
        let query_lower = query.to_lowercase();
        self.search_cache
            .iter()
            .enumerate()
            .filter_map(|(i, cache)| {
                Self::score_server_cached(cache, &query_lower).map(|score| (i, score))
            })
            .collect()
    }

    pub(crate) fn collect_scored_countries(&self, query: &str) -> Vec<(String, u8)> {
        let query_lower = query.to_lowercase();
        self.country_search_cache
            .iter()
            .filter_map(|cache| {
                let code_score = Self::match_score_lower(&cache.code_lower, &query_lower);
                let name_score = Self::match_score_lower(&cache.name_lower, &query_lower);
                let best = code_score.into_iter().chain(name_score).min();
                best.map(|score| (cache.code.clone(), score))
            })
            .collect()
    }

    pub fn update_display_list(&mut self) {
        self.displayed_items.clear();
        let query = &self.search_query;
        let is_searching = !query.is_empty();

        if is_searching {
            let mut scored_servers = self.collect_scored_servers(query);

            // Sort by score (lower is better), then by country name, then by entry IP, then by server name
            // Use cached country_name and entry_ip to avoid repeated lookups
            scored_servers.sort_by(|a, b| {
                let score_cmp = a.1.cmp(&b.1);
                if score_cmp != std::cmp::Ordering::Equal {
                    return score_cmp;
                }
                // Same score: sort by country, then entry IP, then server name
                let cache_a = &self.search_cache[a.0];
                let cache_b = &self.search_cache[b.0];
                let country_cmp = cache_a.country_name.cmp(&cache_b.country_name);
                if country_cmp != std::cmp::Ordering::Equal {
                    return country_cmp;
                }
                let entry_ip_cmp = cache_a.entry_ip.cmp(&cache_b.entry_ip);
                if entry_ip_cmp != std::cmp::Ordering::Equal {
                    return entry_ip_cmp;
                }
                self.all_servers[a.0].name.cmp(&self.all_servers[b.0].name)
            });

            // Build display list, grouping by country (and optionally by entry IP)
            let mut current_country = String::new();
            let mut current_entry_ip = String::new();
            for (server_idx, _score) in scored_servers {
                let server = &self.all_servers[server_idx];
                let entry_ip = &self.search_cache[server_idx].entry_ip;

                if server.exit_country != current_country {
                    current_country = server.exit_country.clone();
                    current_entry_ip.clear();
                    self.displayed_items
                        .push(DisplayItem::CountryHeader(current_country.clone()));
                }
                if self.group_by_entry_ip && *entry_ip != current_entry_ip {
                    current_entry_ip = entry_ip.clone();
                    self.displayed_items.push(DisplayItem::EntryIpHeader(
                        current_country.clone(),
                        current_entry_ip.clone(),
                    ));
                }
                self.displayed_items.push(DisplayItem::Server(server_idx));
            }
        } else {
            // Normal mode: use pre-sorted indices for fast expand/collapse
            let mut current_country = String::new();
            let mut current_entry_ip = String::new();

            for &i in &self.sorted_server_indices {
                let exit_country = &self.all_servers[i].exit_country;
                let entry_ip = &self.search_cache[i].entry_ip;

                if *exit_country != current_country {
                    current_country = exit_country.clone();
                    current_entry_ip.clear();
                    self.displayed_items
                        .push(DisplayItem::CountryHeader(current_country.clone()));
                }

                if self.expanded_countries.contains(&current_country) {
                    if self.group_by_entry_ip {
                        // Group by entry IP: show IP headers and require expansion
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
                    } else {
                        // Flat mode: show servers directly under country
                        self.displayed_items.push(DisplayItem::Server(i));
                    }
                }
            }
        }
    }
}
