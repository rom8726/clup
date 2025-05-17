use serde_json::Value;
use std::net::UdpSocket;
use std::process::Command;
use ureq;
use crate::patroni::patroni::{Patroni, PatroniData};

pub struct Overview {
    pub patroni_srv: Patroni
}

pub struct OverviewData {
    pub hostname: String,
    pub ip: String,
    pub patroni_data: PatroniData,
    pub statuses: Vec<(String, String)>, // (name, "UP"/"DOWN")
    pub errors: Vec<(String, usize)>,    // (name, count)
}

impl Overview {
    pub fn new(patroni_srv: Patroni) -> Self {
        Overview{
            patroni_srv
        }
    }

    pub fn get_overview(&self) -> OverviewData {
        let hostname = self.get_hostname();
        let ip = self.get_local_ip();
        let patroni_data = self.patroni_srv.get_patroni_info();
        let statuses = self.check_services(&["patroni", "haproxy", "pgbouncer", "keepalived"]);
        let errors = self.count_errors(&["patroni", "haproxy", "pgbouncer", "keepalived"]);

        OverviewData {
            hostname,
            ip,
            patroni_data,
            statuses,
            errors,
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

    fn check_services(&self, names: &[&str]) -> Vec<(String, String)> {
        names
            .iter()
            .map(|&s| {
                let out = Command::new("systemctl")
                    .arg("is-active")
                    .arg(format!("{}.service", s))
                    .output();

                let status = match out {
                    Ok(output) if output.status.success() => "UP",
                    _ => "DOWN",
                };
                (s.to_string(), status.to_string())
            })
            .collect()
    }

    fn count_errors(&self, names: &[&str]) -> Vec<(String, usize)> {
        names
            .iter()
            .map(|&s| {
                let out = Command::new("journalctl")
                    .args(["-u", s, "-n", "300", "--no-pager"])
                    .output();

                let count = match out {
                    Ok(output) => {
                        let text = String::from_utf8_lossy(&output.stdout).to_lowercase();
                        text.lines()
                            .filter(|line| line.contains("error") || line.contains("warn"))
                            .count()
                    }
                    _ => 0,
                };
                (s.to_string(), count)
            })
            .collect()
    }
}
