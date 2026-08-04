//! Runtime configuration loaded from environment variables.

use std::net::IpAddr;

#[derive(Debug, Clone)]
pub struct Config {
    pub host: IpAddr,
    pub port: u16,
    pub database_url: String,
    pub device_token_secret: String,
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

        let database_url = std::env::var("DATABASE_URL").expect(
            "DATABASE_URL must be set (e.g. postgres://panagang:panagang@localhost:5433/panagang)",
        );

        let device_token_secret = std::env::var("DEVICE_TOKEN_SECRET")
            .unwrap_or_else(|_| "change-me-dev-only-secret".into());

        Self {
            host,
            port,
            database_url,
            device_token_secret,
        }
    }
}
