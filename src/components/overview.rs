use std::io::{Read, Write};
use crate::patroni::patroni::{ClusterInfo, Patroni};
use std::net::UdpSocket;
use std::process::Command;
use std::os::unix::net::UnixStream;
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
    pub uptime: String,
    pub version: String,
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

                let uptime = Command::new("systemctl")
                    .args([
                        "show",
                        &format!("{svc}.service"),
                        "--property=ActiveEnterTimestamp",
                    ])
                    .output()
                    .map(|o| {
                        String::from_utf8_lossy(&o.stdout)
                            .trim()
                            .split('=')
                            .nth(1) // берём всё после '='
                            .unwrap_or("unknown")
                            .to_string()
                    })
                    .unwrap_or_else(|_| "unknown".into());
                
                let version = Self::detect_version(svc);

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

    fn detect_version(svc: &str) -> String {
        let try_cmd = |arg: &str| -> Option<String> {
            Command::new(svc)
                .arg(arg)
                .output()
                .ok()
                .and_then(|o| {
                    if o.status.success() {
                        Some(
                            String::from_utf8_lossy(&o.stdout)
                                .lines()
                                .next()
                                .unwrap_or("")
                                .trim()
                                .to_string(),
                        )
                    } else {
                        None
                    }
                })
        };

        try_cmd("-v")
            .or_else(|| try_cmd("--version"))
            .unwrap_or_else(|| "-".into())
    }

    pub fn fetch_haproxy_backend_stats(&self) -> (u32, u32) {
        const SOCK_PATH: &str = "/var/run/haproxy/admin.sock";

        let mut stream = match UnixStream::connect(SOCK_PATH) {
            Ok(s) => s,
            Err(_) => return (0, 0),
        };

        if stream.write_all(b"show stat\n").is_err() {
            return (0, 0);
        }

        let mut buf = String::new();
        if stream.read_to_string(&mut buf).is_err() {
            return (0, 0);
        }

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
    }

    pub fn detect_keepalived_vip() -> String {
        let output = Command::new("ip")
            .args(["-o", "-4", "addr", "show", "scope", "global"])
            .output();

        if let Ok(out) = output {
            if out.status.success() {
                for line in String::from_utf8_lossy(&out.stdout).lines() {
                    if line.contains(" secondary ") {
                        if let Some(addr_field) = line.split_whitespace().nth(3) {
                            return addr_field
                                .split('/')
                                .next()
                                .unwrap_or("-")
                                .to_string();
                        }
                    }
                }
            }
        }
        "-".into()
    }
}