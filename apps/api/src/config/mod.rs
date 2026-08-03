//! Runtime configuration loaded from environment variables.

use std::net::IpAddr;

#[derive(Debug, Clone)]
pub struct Config {
    pub host: IpAddr,
    pub port: u16,
}

impl Config {
    pub fn from_env() -> Self {
        let host = std::env::var("API_HOST")
            .unwrap_or_else(|_| "127.0.0.1".into())
            .parse()
            .expect("API_HOST must be a valid IP address");

        let port = std::env::var("API_PORT")
            .unwrap_or_else(|_| "8080".into())
            .parse()
            .expect("API_PORT must be a valid u16");

        Self { host, port }
    }
}
