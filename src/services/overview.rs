use crate::config::Config;
use crate::patroni::patroni::{ClusterInfo, Patroni};
use crate::system;
use std::net::UdpSocket;

pub struct OverviewService {
    pub patroni_client: Patroni,
    pub config: Config,
}

pub struct OverviewData {
    pub hostname: String,
    pub ip: String,
    pub cluster_data: ClusterInfo,
    pub components: Vec<ComponentStatus>,
}

pub struct ComponentStatus {
    pub name: String,
    pub up: bool,
    pub errors: u32,
    pub uptime: String,
    pub version: String,
}

impl OverviewService {
    pub fn new(patroni_client: Patroni, config: Config) -> Self {
        OverviewService { patroni_client, config }
    }

    /// Get overview data including hostname, IP, cluster info, and component statuses
    pub fn get_overview(&self) -> OverviewData {
        let hostname = self.get_hostname();
        let ip = self.get_local_ip();
        let cluster_data = self.patroni_client.get_cluster_info();

        // Convert service names to string slices
        let service_names = self.config.services_list();
        let services: Vec<&str> = service_names.iter().map(|s| s.as_str()).collect();
        let components = self.collect_component_statuses(&services);

        OverviewData {
            hostname,
            ip,
            cluster_data,
            components,
        }
    }

    /// Get the hostname of the current machine
    fn get_hostname(&self) -> String {
        hostname::get()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string()
    }

    /// Get the local IP address
    fn get_local_ip(&self) -> String {
        UdpSocket::bind("0.0.0.0:0")
            .and_then(|sock| {
                sock.connect(&self.config.dns_server)?;
                sock.local_addr()
            })
            .map(|addr| addr.ip().to_string())
            .unwrap_or_else(|_| "unknown".into())
    }

    /// Collect status information for all components
    fn collect_component_statuses(&self, names: &[&str]) -> Vec<ComponentStatus> {
        names
            .iter()
            .map(|&svc| {
                let up = system::is_service_active(svc);
                let errors = system::get_service_errors(svc, 300);
                let uptime = system::get_service_uptime(svc);
                let version = system::detect_service_version(svc);

                ComponentStatus {
                    name: svc.to_string(),
                    up,
                    errors,
                    uptime,
                    version,
                }
            })
            .collect()
    }

    /// Fetch HAProxy backend statistics
    pub fn fetch_haproxy_backend_stats(&self) -> (u32, u32) {
        let result = system::query_haproxy_socket(&self.config.haproxy_socket, "show stat\n");
        
        if let Ok(buf) = result {
            let mut up = 0u32;
            let mut total = 0u32;

            for line in buf.lines().filter(|l| !l.starts_with('#')) {
                let cols: Vec<&str> = line.split(',').collect();
                if cols.len() < 18 {
                    continue;
                }
                let svname = cols[1];
                if svname != "BACKEND" {
                    continue;
                }
                total += 1;
                if cols[17] == "UP" {
                    up += 1;
                }
            }

            (up, total)
        } else {
            (0, 0)
        }
    }

    /// Detect Keepalived VIP (Virtual IP)
    pub fn detect_keepalived_vip() -> String {
        system::detect_keepalived_vip()
    }
}