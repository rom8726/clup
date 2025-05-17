use crate::patroni::patroni::{ClusterInfo, Patroni};
use serde_json::Value;
use std::net::UdpSocket;
use std::process::Command;
use ureq;

pub struct Overview {
    pub patroni_srv: Patroni,
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
}

impl Overview {
    pub fn new(patroni_srv: Patroni) -> Self {
        Overview { patroni_srv }
    }

    pub fn get_overview(&self) -> OverviewData {
        let hostname = self.get_hostname();
        let ip = self.get_local_ip();
        let cluster_data = self.patroni_srv.get_cluster_info();
        let components = self.collect_component_statuses(&[
            "patroni",
            "haproxy",
            "pgbouncer",
            "keepalived",
        ]);

        OverviewData {
            hostname,
            ip,
            cluster_data,
            components,
        }
    }

    fn get_hostname(&self) -> String {
        hostname::get()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string()
    }

    fn get_local_ip(&self) -> String {
        UdpSocket::bind("0.0.0.0:0")
            .and_then(|sock| {
                sock.connect("8.8.8.8:80")?;
                sock.local_addr()
            })
            .map(|addr| addr.ip().to_string())
            .unwrap_or_else(|_| "unknown".into())
    }

    fn collect_component_statuses(&self, names: &[&str]) -> Vec<ComponentStatus> {
        names
            .iter()
            .map(|&svc| {
                let up = Command::new("systemctl")
                    .args(["is-active", &format!("{svc}.service")])
                    .output()
                    .map_or(false, |o| o.status.success());
                
                let errors = Command::new("journalctl")
                    .args(["-u", svc, "-n", "300", "--no-pager"])
                    .output()
                    .map(|o| {
                        String::from_utf8_lossy(&o.stdout)
                            .to_lowercase()
                            .lines()
                            .filter(|l| l.contains("error") || l.contains("fatal"))
                            .count() as u32
                    })
                    .unwrap_or(0);

                ComponentStatus {
                    name: svc.to_string(),
                    up,
                    errors,
                }
            })
            .collect()
    }
}
