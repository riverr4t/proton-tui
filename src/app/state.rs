use std::time::Instant;

use crate::models::LogicalServer;

#[derive(PartialEq, Clone, Copy)]
pub enum SplitFocus {
    Countries,
    Servers,
}

#[derive(PartialEq, Clone, Copy)]
pub enum FocusPanel {
    Favorites,
    Main,
}

#[derive(Clone, PartialEq)]
pub enum DisplayItem {
    CountryHeader(String),
    EntryIpHeader(String, String), // (country_code, entry_ip)
    RegionHeader(String, String),  // (country_code, region_code)
    Server(usize),                 // Index in all_servers
}

pub struct ConnectionStatus {
    pub server_name: String,
    pub interface: String,
    pub connected_at: Instant,
    pub rx_bytes: u64,
    pub tx_bytes: u64,
    // Speed tracking
    pub prev_rx_bytes: u64,
    pub prev_tx_bytes: u64,
    pub last_sample_time: Instant,
    pub rx_speed: f64, // bytes per second
    pub tx_speed: f64,
    pub rx_history: Vec<u64>, // speed samples for sparkline (max 60)
    pub tx_history: Vec<u64>,
}

#[derive(Clone, Default, PartialEq)]
pub struct ServerFilter {
    pub max_load: Option<i32>,
    pub features: Option<i32>,
    pub min_tier: Option<i32>,
    pub max_tier: Option<i32>,
    pub online_only: bool,
    pub favorites_only: bool,
}

impl ServerFilter {
    pub fn matches(&self, server: &LogicalServer) -> bool {
        if let Some(max) = self.max_load {
            if server.load > max {
                return false;
            }
        }
        if let Some(feat_mask) = self.features {
            if server.features & feat_mask != feat_mask {
                return false;
            }
        }
        if let Some(min) = self.min_tier {
            if server.tier < min {
                return false;
            }
        }
        if let Some(max) = self.max_tier {
            if server.tier > max {
                return false;
            }
        }
        if self.online_only && server.status != 1 {
            return false;
        }
        true
    }

    pub fn is_active(&self) -> bool {
        self.max_load.is_some()
            || self.features.is_some()
            || self.min_tier.is_some()
            || self.max_tier.is_some()
            || self.online_only
            || self.favorites_only
    }

    pub fn active_count(&self) -> usize {
        let mut count = 0;
        if self.max_load.is_some() {
            count += 1;
        }
        if self.features.is_some() {
            count += 1;
        }
        if self.min_tier.is_some() || self.max_tier.is_some() {
            count += 1;
        }
        if self.online_only {
            count += 1;
        }
        if self.favorites_only {
            count += 1;
        }
        count
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum SortField {
    Name,
    Load,
    Score,
    Country,
}

impl SortField {
    pub fn label(&self) -> &str {
        match self {
            SortField::Name => "Name",
            SortField::Load => "Load",
            SortField::Score => "Score",
            SortField::Country => "Country",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            SortField::Name => SortField::Load,
            SortField::Load => SortField::Score,
            SortField::Score => SortField::Country,
            SortField::Country => SortField::Name,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

impl SortDirection {
    pub fn toggle(&self) -> Self {
        match self {
            SortDirection::Ascending => SortDirection::Descending,
            SortDirection::Descending => SortDirection::Ascending,
        }
    }

    pub fn indicator(&self) -> &str {
        match self {
            SortDirection::Ascending => "↑",
            SortDirection::Descending => "↓",
        }
    }
}
