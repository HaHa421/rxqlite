#![allow(clippy::uninlined_format_args)]
#![deny(unused_qualifications)]
//#![deny(unused_crate_dependencies)]
#![deny(unused_extern_crates)]
#![deny(warnings)]

use std::fmt::Display;
use std::io::Cursor;
use std::path::Path;
use std::sync::Arc;

use openraft::Config;
use openraft::TokioRuntime;
use tokio::net::TcpListener;
use tokio::task;

use crate::app::App;
use crate::network::api;
use crate::network::management;
use crate::network::Network;
use crate::store::new_storage;
use crate::store::Request;
use crate::store::Response;

pub mod app;
pub mod client;
pub mod network;
pub mod sqlite_store;
use sqlite_store as store;
use warp::Filter;
use std::net::SocketAddr;
use std::str::FromStr;

pub type NodeId = u64;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, Default)]
pub struct Node {
    pub rpc_addr: String,
    pub api_addr: String,
}

impl Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Node {{ rpc_addr: {}, api_addr: {} }}", self.rpc_addr, self.api_addr)
    }
}

pub type SnapshotData = Cursor<Vec<u8>>;

openraft::declare_raft_types!(
    pub TypeConfig:
        D = Request,
        R = Response,
        NodeId = NodeId,
        Node = Node,
        Entry = openraft::Entry<TypeConfig>,
        SnapshotData = SnapshotData,
        AsyncRuntime = TokioRuntime
);

pub mod typ {
    use openraft::error::Infallible;

    use crate::Node;
    use crate::NodeId;
    use crate::TypeConfig;

    pub type Entry = openraft::Entry<TypeConfig>;

    pub type RaftError<E = Infallible> = openraft::error::RaftError<NodeId, E>;
    pub type RPCError<E = Infallible> = openraft::error::RPCError<NodeId, Node, RaftError<E>>;

    pub type ClientWriteError = openraft::error::ClientWriteError<NodeId, Node>;
    pub type CheckIsLeaderError = openraft::error::CheckIsLeaderError<NodeId, Node>;
    pub type ForwardToLeader = openraft::error::ForwardToLeader<NodeId, Node>;
    pub type InitializeError = openraft::error::InitializeError<NodeId, Node>;

    pub type ClientWriteResponse = openraft::raft::ClientWriteResponse<TypeConfig>;
}

pub type ExampleRaft = openraft::Raft<TypeConfig>;

//type Server = tide::Server<Arc<App>>;

fn with_app(app: Arc<App>) -> impl Filter<Extract = (Arc<App>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || app.clone())
}

pub async fn start_example_raft_node<P>(
    node_id: NodeId,
    base_dir: P,
    http_addr: String,
    rpc_addr: String,
) -> std::io::Result<()>
where
    P: AsRef<Path>,
{
    std::fs::create_dir_all(&base_dir)?;
    let rocksdb_dir = base_dir.as_ref().join("rocksb");
    let sqlite_path = base_dir.as_ref().join("sqlite.db");
    
    // Create a configuration for the raft instance.
    let config = Config {
        heartbeat_interval: 250,
        election_timeout_min: 299,
        ..Default::default()
    };
    
    let config = Arc::new(config.validate().unwrap());
    
    let (log_store, state_machine_store) = new_storage(&rocksdb_dir,&sqlite_path).await?;
    
    let sqlite_and_path = state_machine_store.data.sqlite_and_path.clone();
    
    // Create the network layer that will connect and communicate the raft instances and
    // will be used in conjunction with the store created above.
    let network = Network {};

    // Create a local raft instance.
    let raft = openraft::Raft::new(node_id, config.clone(), network, log_store, state_machine_store).await.unwrap();
    
    let app = Arc::new(App {
        id: node_id,
        api_addr: http_addr.clone(),
        rpc_addr: rpc_addr.clone(),
        raft,
        //key_values: kvs,
        sqlite_and_path,
        config,
    });

    let echo_service = Arc::new(network::raft::Raft::new(app.clone()));
    
    let server = toy_rpc::Server::builder().register(echo_service).build();
    
    let listener = TcpListener::bind(rpc_addr).await.unwrap();
    
    let handle = task::spawn(async move {
        server.accept_websocket(listener).await.unwrap();
    });
    
    
    let execute_query = warp::post()
        .and(warp::path!("api" / "sql"))
        .and(warp::body::json())
        .and(with_app(app.clone()))
        .and_then(
          |arg0 : Message, arg1: Arc<App>| api::sql(arg0, arg1)
        )
        ;
        
    let management_add_learner = warp::post()
        .and(warp::path!("cluster" / "add-learner"))
        .and(warp::body::json())
        .and(with_app(app.clone()))
      .and_then(management::add_learner);
    
    let management_change_membership = warp::post()
        .and(warp::path!("cluster" / "change-membership"))
        .and(warp::body::json())
        .and(with_app(app.clone()))
      .and_then(management::change_membership);
    
    let management_init = warp::post()
        .and(warp::path!("cluster" / "init"))
        .and(warp::body::json())
        .and(with_app(app.clone()))
      .and_then(management::init);
      
    let management_metrics = warp::get()
        .and(warp::path!("cluster" / "metrics"))
        .and(with_app(app.clone()))
        .and_then(management::metrics);

    let management_snapshot = warp::post()
        .and(warp::path!("cluster" / "snapshot"))
        .and(warp::body::json())
        .and(with_app(app.clone()))
      .and_then(management::snapshot);
      
    let routes = execute_query
      .or(management_add_learner)
      .or(management_change_membership)
      .or(management_init)
      .or(management_metrics)
      .or(management_snapshot);

    let _server = tokio::spawn(async move {
            warp::serve(routes)
                .run(SocketAddr::from_str(&http_addr).unwrap())
                .await;
    });
    
    
    _ = handle.await;
    Ok(())
}

pub use rxqlite_common::{Message,MessageResponse,Value};

#[derive(Debug,Default,Clone,Copy,PartialEq,Eq)]
pub enum Scheme {
  #[default]
  HTTP,
  HTTPS,
}

#[derive(Default,Debug,Clone)]
pub struct ConnectOptions {
  pub scheme: Scheme,
  pub leader_id: NodeId,
  pub leader_host: String,
  pub leader_port: u16,
  pub accept_invalid_cert: bool,
}

pub type RaftSqliteError = anyhow::Error;

impl ConnectOptions {
  pub async fn connect(&self)->Result<client::ExampleClient,RaftSqliteError> {
    Ok(client::ExampleClient::with_options(self))
  }
}

pub use client::ExampleClient as Connection;

pub use rxqlite_common::FromValueRef;