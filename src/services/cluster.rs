use crate::patroni::patroni::{ClusterInfo, Patroni};

pub struct ClusterService {
    pub patroni_client: Patroni,
}

impl ClusterService {
    pub fn new(patroni_client: Patroni) -> Self {
        ClusterService { patroni_client }
    }

    /// Get information about the cluster
    pub fn get_cluster_info(&self) -> ClusterInfo {
        let mut info = self.patroni_client.get_cluster_info();
        info.members.sort_by_key(|node| node.name.clone());

        info
    }
}