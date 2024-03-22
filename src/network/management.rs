//use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::sync::Arc;

use openraft::error::Infallible;
use openraft::RaftMetrics;

use warp::reply;
use crate::app::App;
use crate::Node;
use crate::NodeId;
//use rxqlite_common::{RSQliteNodeConfig};

//use crate::TypeConfig;
use serde::Deserialize;
// --- Cluster management

#[derive(Deserialize)]
pub struct Empty {}

/// Add a node as **Learner**.
///
/// A Learner receives log replication from the leader but does not vote.
/// This should be done before adding a node as a member into the cluster
/// (by calling `change-membership`)
pub async fn add_learner(
    (node_id, api_addr, rpc_addr): (NodeId, String, String),
    app: Arc<App>) -> Result<impl warp::Reply, std::convert::Infallible> {
    let node = Node { 
      rpc_addr, 
      api_addr,
      //tls_config: None,
    };
    let res = app.raft.add_learner(node_id, node, true).await;
    Ok(reply::json(&res))
}

/// Changes specified learners to members, or remove members.
pub async fn change_membership(
    body: BTreeSet<NodeId>,
    app: Arc<App>) -> Result<impl warp::Reply, std::convert::Infallible> {
    let res = app.raft.change_membership(body, false).await;
    Ok(reply::json(&res))
}

/*
/// Initialize a single-node cluster.
pub async fn init(_opts: RSQliteNodeConfig, app: Arc<App>) -> Result<impl warp::Reply, std::convert::Infallible> {
    let mut nodes = BTreeMap::new();
    let node = Node { 
      rpc_addr: app.rpc_addr.clone(), 
      api_addr: app.api_addr.clone(),
      //tls_config: None,
    };

    nodes.insert(app.id, node);
    let res = app.raft.initialize(nodes).await;
    Ok(reply::json(&res))
}
*/
/// Get the latest metrics of the cluster
pub async fn metrics(app: Arc<App>) -> Result<impl warp::Reply, std::convert::Infallible> {
    let metrics = app.raft.metrics().borrow().clone();
    let res: Result<RaftMetrics<NodeId, Node>, Infallible> = Ok(metrics);
    Ok(reply::json(&res))
}

pub async fn snapshot(_: Empty, app: Arc<App>) -> Result<impl warp::Reply, std::convert::Infallible> {
    let res = app.raft.trigger().snapshot().await;
    Ok(reply::json(&res))
}
