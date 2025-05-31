use crate::patroni::patroni::{ClusterInfo, Patroni};
use anyhow::{Result, anyhow};

pub struct ActionsService {
    pub patroni_client: Patroni,
}

#[derive(Debug, Clone)]
pub enum Action {
    Switchover,
    Restart,
    Reinitialize,
    PauseCluster,
    ResumeCluster,
}

impl Action {
    pub fn as_str(&self) -> &'static str {
        match self {
            Action::Switchover => "Switchover",
            Action::Restart => "Restart Node",
            Action::Reinitialize => "Reinitialize Node",
            Action::PauseCluster => "Pause Cluster",
            Action::ResumeCluster => "Resume Cluster",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Action::Switchover => "Change the leader node in the cluster",
            Action::Restart => "Restart a specific node in the cluster",
            Action::Reinitialize => "Reinitialize a specific node in the cluster",
            Action::PauseCluster => "Pause automatic failover in the cluster",
            Action::ResumeCluster => "Resume automatic failover in the cluster",
        }
    }

    pub fn is_destructive(&self) -> bool {
        match self {
            Action::Switchover => true,
            Action::Restart => true,
            Action::Reinitialize => true,
            Action::PauseCluster => true,
            Action::ResumeCluster => false,
        }
    }

    pub fn all() -> Vec<Action> {
        vec![
            Action::Switchover,
            Action::Restart,
            Action::Reinitialize,
            Action::PauseCluster,
            Action::ResumeCluster,
        ]
    }
}

impl ActionsService {
    pub fn new(patroni_client: Patroni) -> Self {
        ActionsService { patroni_client }
    }

    pub fn get_cluster_info(&self) -> ClusterInfo {
        self.patroni_client.get_cluster_info()
    }

    pub fn switchover(&self, leader: &str, candidate: &str) -> Result<()> {
        let url = format!("{}/switchover", self.patroni_client.base_url());
        let body = format!(r#"{{"leader": "{}", "candidate": "{}"}}"#, leader, candidate);

        match ureq::post(&url)
            .set("Content-Type", "application/json")
            .send_string(&body)
        {
            Ok(_) => Ok(()),
            Err(e) => Err(anyhow!("Failed to perform switchover: {}", e)),
        }
    }

    pub fn restart_node(&self, node: &str) -> Result<()> {
        let url = format!("{}/restart", self.patroni_client.base_url());
        let body = format!(r#"{{"restart": true}}"#);

        match ureq::post(&url)
            .set("Content-Type", "application/json")
            .send_string(&body)
        {
            Ok(_) => Ok(()),
            Err(e) => Err(anyhow!("Failed to restart node {}: {}", node, e)),
        }
    }

    pub fn reinitialize_node(&self, node: &str) -> Result<()> {
        let url = format!("{}/reinitialize", self.patroni_client.base_url());
        let body = format!(r#"{{"reinitialize": true}}"#);

        match ureq::post(&url)
            .set("Content-Type", "application/json")
            .send_string(&body)
        {
            Ok(_) => Ok(()),
            Err(e) => Err(anyhow!("Failed to reinitialize node {}: {}", node, e)),
        }
    }

    pub fn pause_cluster(&self) -> Result<()> {
        let url = format!("{}/pause", self.patroni_client.base_url());
        let body = r#"{"paused": true}"#;

        match ureq::patch(&url)
            .set("Content-Type", "application/json")
            .send_string(body)
        {
            Ok(_) => Ok(()),
            Err(e) => Err(anyhow!("Failed to pause cluster: {}", e)),
        }
    }

    pub fn resume_cluster(&self) -> Result<()> {
        let url = format!("{}/pause", self.patroni_client.base_url());
        let body = r#"{"paused": false}"#;

        match ureq::patch(&url)
            .set("Content-Type", "application/json")
            .send_string(body)
        {
            Ok(_) => Ok(()),
            Err(e) => Err(anyhow!("Failed to resume cluster: {}", e)),
        }
    }
}