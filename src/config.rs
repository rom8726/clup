use clap::Parser;
use std::time::Duration;

/// clup - CLI application for monitoring a Patroni PostgreSQL cluster
#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Config {
    /// Patroni API address
    #[arg(short, long, default_value = "127.0.0.1:8008")]
    pub patroni_addr: String,

    /// DNS server for IP detection
    #[arg(long, default_value = "8.8.8.8:80")]
    pub dns_server: String,

    /// HAProxy socket path
    #[arg(long, default_value = "/var/run/haproxy/admin.sock")]
    pub haproxy_socket: String,

    /// Maximum replication lag in seconds
    #[arg(long, default_value = "10")]
    pub max_replication_lag_secs: u64,

    /// Services to monitor (comma-separated)
    #[arg(long, default_value = "patroni,haproxy,pgbouncer,keepalived")]
    pub services: String,
}

impl Config {
    /// Parse command-line arguments into Config
    pub fn new() -> Self {
        Config::parse()
    }

    /// Get the maximum replication lag in microseconds
    pub fn max_replication_lag_us(&self) -> u64 {
        self.max_replication_lag_secs * 1_000_000
    }

    /// Get the list of services to monitor
    pub fn services_list(&self) -> Vec<String> {
        self.services
            .split(',')
            .map(|s| s.trim().to_string())
            .collect()
    }
}