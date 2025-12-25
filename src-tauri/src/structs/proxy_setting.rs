use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ProxyInfo {
    pub protocal: String, // http、socks4、socks5
    pub host: String,
    pub port: u16,
}

impl Default for ProxyInfo {
    fn default() -> Self {
        Self {
            protocal: String::from("http"),
            host: String::new(),
            port: 0,
        }
    }
}
