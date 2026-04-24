use serde::{Deserialize, Serialize};

fn default_protocol() -> String {
    "http".to_string()
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct ProxyInfo {
    #[serde(default = "default_protocol")]
    pub protocol: String, // http、socks4、socks5
    pub host: String,
    pub port: u16,
}

impl Default for ProxyInfo {
    fn default() -> Self {
        Self {
            protocol: default_protocol(),
            host: String::new(),
            port: 0,
        }
    }
}
