use crate::patroni::patroni::{ClusterInfo, Patroni};

pub struct Cluster {
    pub patroni_srv: Patroni
}

impl Cluster {
    pub fn new(patroni_srv: Patroni) -> Self {
        Cluster {
            patroni_srv
        }
    }

    pub fn get_cluster_info(&self) -> ClusterInfo {
        self.patroni_srv.get_cluster_info()
    }
}