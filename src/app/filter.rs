use super::state::{ServerFilter, SortDirection, SortField};
use super::App;

/// Load threshold options for the filter popup
pub const LOAD_THRESHOLDS: &[i32] = &[30, 50, 70, 90];

/// Feature bitmask constants
pub const FEATURE_SC: i32 = 1;
pub const FEATURE_TOR: i32 = 2;
pub const FEATURE_P2P: i32 = 4;
pub const FEATURE_STR: i32 = 8;

impl App {
    pub fn toggle_sort_field(&mut self) {
        self.sort_field = self.sort_field.next();
        self.resort_and_refresh();
    }

    pub fn toggle_sort_direction(&mut self) {
        self.sort_direction = self.sort_direction.toggle();
        self.resort_and_refresh();
    }

    pub fn cycle_load_filter(&mut self) {
        self.active_filter.max_load = match self.active_filter.max_load {
            None => Some(LOAD_THRESHOLDS[0]),
            Some(current) => {
                let pos = LOAD_THRESHOLDS.iter().position(|&t| t == current);
                match pos {
                    Some(i) if i + 1 < LOAD_THRESHOLDS.len() => Some(LOAD_THRESHOLDS[i + 1]),
                    _ => None,
                }
            }
        };
        self.refresh_after_filter();
    }

    pub fn toggle_feature_filter(&mut self, feature: i32) {
        let current = self.active_filter.features.unwrap_or(0);
        let new_mask = current ^ feature;
        self.active_filter.features = if new_mask == 0 { None } else { Some(new_mask) };
        self.refresh_after_filter();
    }

    pub fn toggle_online_filter(&mut self) {
        self.active_filter.online_only = !self.active_filter.online_only;
        self.refresh_after_filter();
    }

    pub fn reset_filters(&mut self) {
        self.active_filter = ServerFilter::default();
        self.sort_field = SortField::Name;
        self.sort_direction = SortDirection::Ascending;
        self.resort_and_refresh();
    }

    fn resort_and_refresh(&mut self) {
        let sort_field = self.sort_field;
        let sort_dir = self.sort_direction;
        let group_by_ip = self.group_by_entry_ip;
        let servers = &self.all_servers;
        let cache = &self.search_cache;

        self.sorted_server_indices.sort_by(|&a, &b| {
            let ord = match sort_field {
                SortField::Name => servers[a].name.cmp(&servers[b].name),
                SortField::Load => servers[a].load.cmp(&servers[b].load),
                SortField::Score => servers[a]
                    .score
                    .partial_cmp(&servers[b].score)
                    .unwrap_or(std::cmp::Ordering::Equal),
                SortField::Country => {
                    let region_cmp = if group_by_ip {
                        match (&cache[a].region_name, &cache[b].region_name) {
                            (Some(a_r), Some(b_r)) => a_r.cmp(b_r),
                            (Some(_), None) => std::cmp::Ordering::Less,
                            (None, Some(_)) => std::cmp::Ordering::Greater,
                            (None, None) => std::cmp::Ordering::Equal,
                        }
                    } else {
                        std::cmp::Ordering::Equal
                    };
                    cache[a]
                        .country_name
                        .cmp(&cache[b].country_name)
                        .then(region_cmp)
                        .then(cache[a].entry_ip.cmp(&cache[b].entry_ip))
                        .then(servers[a].name.cmp(&servers[b].name))
                }
            };
            match sort_dir {
                SortDirection::Ascending => ord,
                SortDirection::Descending => ord.reverse(),
            }
        });

        self.refresh_after_filter();
    }

    pub fn refresh_after_filter(&mut self) {
        if self.split_view {
            self.update_server_list_for_selected_country();
        } else {
            self.update_display_list();
            if !self.displayed_items.is_empty() {
                if self.state.selected().is_none()
                    || self.state.selected().unwrap() >= self.displayed_items.len()
                {
                    self.state.select(Some(0));
                }
            } else {
                self.state.select(None);
            }
        }
    }
}
