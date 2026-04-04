use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct LogicalServersResponse {
    pub logical_servers: Vec<LogicalServer>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct LogicalServer {
    #[serde(alias = "ID")]
    pub id: String,
    pub name: String,
    pub entry_country: String,
    pub exit_country: String,
    pub tier: i32,
    pub features: i32,
    pub score: f64,
    pub load: i32,
    pub status: i32,
    pub city: String,
    pub servers: Vec<ServerInstance>,
    pub domain: Option<String>,
}

#[allow(dead_code)]
impl LogicalServer {
    pub fn is_secure_core(&self) -> bool {
        self.features & 1 != 0
    }

    pub fn is_tor(&self) -> bool {
        self.features & 2 != 0
    }

    pub fn is_p2p(&self) -> bool {
        self.features & 4 != 0
    }

    pub fn is_streaming(&self) -> bool {
        self.features & 8 != 0
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ServerInstance {
    #[serde(alias = "ID")]
    pub id: String,
    #[serde(alias = "EntryIP")]
    pub entry_ip: String,
    #[serde(alias = "ExitIP")]
    pub exit_ip: String,
    pub domain: String,
    #[serde(rename = "X25519PublicKey")]
    pub x25519_public_key: String,
    pub label: Option<String>,
}
