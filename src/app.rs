use std::sync::Arc;

use openraft::Config;
use tokio::sync::RwLock;

use crate::sqlite_store::*;
use crate::ExampleRaft;
use crate::NodeId;
// Representation of an application state. This struct can be shared around to share
// instances of raft, store and more.
pub struct App {
    pub id: NodeId,
    pub api_addr: String,
    pub rpc_addr: String,
    pub raft: ExampleRaft,
    //pub key_values: Arc<RwLock<BTreeMap<String, String>>>,
    pub sqlite_and_path: Arc<RwLock<SqliteAndPath>>,
    pub config: Arc<Config>,
}
