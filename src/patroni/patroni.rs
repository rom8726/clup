use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct Patroni {
    pub addr: String,
}

#[derive(Debug, Clone)]
pub struct ClusterInfo {
    pub scope: String,
    pub node_name: String,
    pub leader_node_name: String,
    pub members: Vec<NodeStatus>,
    pub members_map: HashMap<String, NodeStatus>,
    pub patroni_data: PatroniData,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NodesStatus {
    pub scope: String,
    pub members: Vec<NodeStatus>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NodeStatus {
    pub name: String,
    pub role: String,
    pub state: String,
    pub host: String,
    pub lag: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct PatroniData {
    pub role: String,
    pub state: String,
    pub scope: String,
    pub node_name: String,
}

impl Patroni {
    pub fn new(addr: String) -> Self {
        Patroni { addr }
    }

    fn base_url(&self) -> String {
        "http://".to_string() + self.addr.clone().as_str()
    }

    pub fn get_cluster_info(&self) -> ClusterInfo {
        let nodes = self.get_cluster_nodes();
        let mut members = HashMap::new();
        let mut leader_node_name: String = "-".to_string();

        for node in nodes.clone() {
            members.insert(node.name.clone(), node.clone());
            if node.role == "leader" {
                leader_node_name = node.name.clone();
            }
        }

        let patroni_info = self.get_patroni_info();

        ClusterInfo {
            scope: patroni_info.scope.clone(),
            node_name: patroni_info.node_name.clone(),
            leader_node_name: leader_node_name,
            members: nodes,
            members_map: members,
            patroni_data: patroni_info,
        }
    }

    pub fn get_cluster_nodes(&self) -> Vec<NodeStatus> {
        let url = self.base_url() + "/cluster";
        let resp = ureq::get(url.as_str()).call();

        if let Ok(response) = resp {
            if let Ok(mut data) = response.into_json::<NodesStatus>() {
                data.members.sort_by_key(|node| node.name.clone());

                return data.members;
            }
        }

        vec![]
    }

    pub fn get_patroni_info(&self) -> PatroniData {
        match ureq::get(self.base_url().as_str()).call() {
            Ok(resp) => {
                if let Ok(json) = resp.into_json::<Value>() {
                    return self.parse_patroni_json(json);
                }
            }
            Err(ureq::Error::Status(503, resp)) => {
                if let Ok(json) = resp.into_json::<Value>() {
                    return self.parse_patroni_json(json);
                }
            }
            Err(_) => {}
        }

        PatroniData {
            role: "-".to_string(),
            state: "-".to_string(),
            scope: "-".to_string(),
            node_name: "-".to_string(),
        }
    }

    pub fn parse_patroni_json(&self, json: Value) -> PatroniData {
        PatroniData {
            role: json["role"].as_str().unwrap_or("-").to_string(),
            state: json["state"].as_str().unwrap_or("-").to_string(),
            scope: json["patroni"]["scope"].as_str().unwrap_or("-").to_string(),
            node_name: json["patroni"]["name"].as_str().unwrap_or("-").to_string(),
        }
    }
}
