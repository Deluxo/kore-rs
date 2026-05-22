pub mod manager;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Host {
    pub id: String,
    pub name: String,
    pub address: String,
    pub port: u16,
    pub mac_address: Option<String>,
    pub wol_port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
    pub use_tls: bool,
}

impl Host {
    pub fn new(name: String, address: String, port: u16) -> Self {
        Self {
            id: uuid_simple(),
            name,
            address,
            port,
            mac_address: None,
            wol_port: 9,
            username: None,
            password: None,
            use_tls: false,
        }
    }

    pub fn new_with_credentials(
        name: String,
        address: String,
        port: u16,
        username: Option<String>,
        password: Option<String>,
    ) -> Self {
        Self {
            id: uuid_simple(),
            name,
            address,
            port,
            mac_address: None,
            wol_port: 9,
            username,
            password,
            use_tls: false,
        }
    }

    pub fn url(&self) -> String {
        let scheme = if self.use_tls { "https" } else { "http" };
        match (&self.username, &self.password) {
            (Some(user), Some(pass)) => format!("{}://{}:{}@{}:{}", scheme, user, pass, self.address, self.port),
            (Some(user), None) => format!("{}://{}@{}:{}", scheme, user, self.address, self.port),
            _ => format!("{}://{}:{}", scheme, self.address, self.port),
        }
    }
}

fn uuid_simple() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{:x}", timestamp)
}
