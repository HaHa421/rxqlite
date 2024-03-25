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


pub mod cipher;
use cipher::{EncryptData};
pub use cipher::NoEncrypt;

use sqlite_store as store;
use warp::Filter;
use std::net::SocketAddr;
use std::str::FromStr;
use rxqlite_common::{RSQliteNodeTlsConfig};
pub use rxqlite_common::RSQliteClientTlsConfig;

use std::collections::{BTreeMap,BTreeSet};

//use rustls_pemfile::{certs, rsa_private_keys};
use rustls::{ServerConfig, ConfigBuilder, server::WantsServerCert};
use std::fs::File;
use std::io::BufReader;
//use std::io;
use serde::{Serialize,Deserialize};

#[cfg(feature = "sqlcipher")]
use crate::cipher::ring::Aes256GcmEncryptor;
#[cfg(feature = "sqlcipher")]
use ring::digest;
#[cfg(feature = "sqlcipher")]
use base64::{engine::general_purpose::URL_SAFE, Engine as _};

/*
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::time::{timeout, Duration};
use futures::future::poll_fn;
*/

use toy_rpc_ha421 as toy_rpc_ha421;

pub type NodeId = u64;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, Default)]
pub struct Node {
    pub rpc_addr: String,
    pub api_addr: String,
    //pub tls_config: Option<RSQliteNodeTlsConfig>,
}
/*
impl Node {
  pub fn new(rpc_addr: String,
    api_addr: String,)->Self {
    Self {
      rpc_addr,
      api_addr,
      tls_config: None,
    }
  }
  pub fn with_tls_config(mut self,
    tls_config:RSQliteNodeTlsConfig)->Self {
    self.tls_config = Some(tls_config);
    self
  }
}
*/
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
    
    pub type RaftMetrics = openraft::metrics::RaftMetrics<NodeId, Node>;
    
    pub type RaftServerMetrics = openraft::metrics::RaftServerMetrics<NodeId, Node>;
}

pub type ExampleRaft = openraft::Raft<TypeConfig>;



fn with_app(app: Arc<App>) -> impl Filter<Extract = (Arc<App>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || app.clone())
}
/*
async fn poll_future<T>(future: Pin<&mut T>, duration: Duration) -> Poll<T::Output> 
where
    T: Future,
{
    let res = timeout(duration, poll_fn(|cx| Future::poll(future, cx))).await;
    match res {
        Ok(poll) => poll,
        Err(_) => Poll::Pending,
    }
}
*/

#[derive(Serialize,Deserialize,Clone)]
pub struct InstanceParams {
  pub http_addr: String,
  pub rpc_addr: String,
  tls_config: Option<RSQliteNodeTlsConfig>,
}

async fn init_rxqlite<P>(
  node_id: NodeId,
  base_dir: P,
  instance_params: InstanceParams) -> anyhow::Result<(Arc<App>,tokio::task::JoinHandle<()>)>
where
    P: AsRef<Path>,
{
  let (_key,encrypt_data) : (Option<String>,Option<Arc<Box<dyn EncryptData>>>) =
  {
    #[cfg(feature = "sqlcipher")] 
    {
      
      if let Some(tls_config) = instance_params.tls_config.as_ref() {
        let private_key_bytes =
        rustls_pemfile::pkcs8_private_keys(&mut BufReader::new(&mut File::open(&tls_config.key_path)?))?.remove(0)
            ;
            
        let hashed_key = digest::digest(&digest::SHA256, &private_key_bytes);
        
        let hashed_key = URL_SAFE.encode(hashed_key.as_ref());
        
        let private_key = rustls::pki_types::PrivatePkcs8KeyDer::from(private_key_bytes);
        
        let encrypt_data = Aes256GcmEncryptor::new(&private_key);
        
        
        
        
        
        (Some(hashed_key),Some(Arc::new(Box::new(encrypt_data))))
      } else {
        (None,None)
      }
    }
    #[cfg(not(feature = "sqlcipher"))] 
    {
      (None,None)
    }
  };
  let rocksdb_dir = base_dir.as_ref().join("rocksdb");
  let sqlite_path = base_dir.as_ref().join("sqlite.db");
    
    // Create a configuration for the raft instance.
    let config = Config {
        heartbeat_interval: 250,
        election_timeout_min: 299,
        ..Default::default()
    };
    
    let config = Arc::new(config.validate().unwrap());
    
    let (log_store, state_machine_store) = new_storage(&rocksdb_dir,
      &sqlite_path,
#[cfg(feature = "sqlcipher")] 
      _key,
      encrypt_data,
      ).await?;
    
    let sqlite_and_path = state_machine_store.data.sqlite_and_path.clone();
    
    // Create the network layer that will connect and communicate the raft instances and
    // will be used in conjunction with the store created above.
    let network = Network {
      tls_config: instance_params.tls_config.clone(),
    };

    // Create a local raft instance.
    let raft = openraft::Raft::new(node_id, config.clone(), network, log_store, state_machine_store).await.unwrap();
    
    let app = Arc::new(App {
        id: node_id,
        api_addr: instance_params.http_addr.clone(),
        rpc_addr: instance_params.rpc_addr.clone(),
        raft,
        //key_values: kvs,
        sqlite_and_path,
        config,
    });
    let echo_service = Arc::new(network::raft::Raft::new(app.clone()));
    
    let mut server_builder = toy_rpc_ha421::Server::builder();
    let handle = if let Some(tls_config) = instance_params.tls_config.as_ref() {
      let certs = rustls_pemfile::certs(&mut BufReader::new(&mut File::open(&tls_config.cert_path)?))?
        .into_iter().map(|x|rustls::pki_types::CertificateDer::from(x)).collect::<Vec<_>>();
      let mut private_keys =
        rustls_pemfile::pkcs8_private_keys(&mut BufReader::new(&mut File::open(&tls_config.key_path)?))?
            ;
            /*
      let certs = load_certs(&tls_config.cert_path).unwrap();
      let mut keys = load_keys(&tls_config.key_path).unwrap();
            */
      
      let config_builder : ConfigBuilder<ServerConfig, WantsServerCert> = ServerConfig::builder()
        //.with_safe_defaults()
        .with_no_client_auth();
        
      let config = config_builder.with_single_cert(certs, rustls::pki_types::PrivatePkcs8KeyDer::from(private_keys.remove(0)).into())?;
      /*
      if tls_config.accept_invalid_cert {
        config.dangerous().set_certificate_verifier(Arc::new(AllowAnyCertVerifier));
      }
      */
      server_builder=server_builder.register(echo_service);
      let server = server_builder.build();
      
      let listener = TcpListener::bind(instance_params.rpc_addr.clone()).await?;
      
      let handle = task::spawn(async move {
          server.accept_with_tls_config(listener, config).await.unwrap();
      });
      handle  
    } else {
      server_builder=server_builder.register(echo_service);
      let server = server_builder.build();
      
      let listener = TcpListener::bind(instance_params.rpc_addr.clone()).await?;
      
      let handle = task::spawn(async move {
          server.accept_websocket(listener).await.unwrap();
      });
      handle
    };
    
  let execute_consistent_query = warp::post()
        .and(warp::path!("api" / "sql-consistent"))
        .and(warp::body::json())
        .and(with_app(app.clone()))
        .and_then(
          |arg0 : Message, arg1: Arc<App>| api::sql_consistent(arg0, arg1)
        )
        ;
    
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
      .or(execute_consistent_query)
      .or(management_add_learner)
      .or(management_change_membership)
      //.or(management_init)
      .or(management_metrics)
      .or(management_snapshot);
    
    let instance_params_=instance_params.clone();
    let _server = tokio::spawn(async move {
            if let Some(tls_config) = instance_params_.tls_config {
              warp::serve(routes)
                .tls()
                .cert_path(&tls_config.cert_path)
                .key_path(&tls_config.key_path)
                .run(SocketAddr::from_str(&instance_params_.http_addr).unwrap())
                .await;
              
            } else {
              warp::serve(routes)
                .run(SocketAddr::from_str(&instance_params_.http_addr).unwrap())
                .await;
            }
    });
    
  Ok((app,handle))
}

pub async fn init_example_raft_node<P>(
    node_id: NodeId,
    base_dir: P,
    leader: bool,
    http_addr: Option<String>,
    rpc_addr: Option<String>,
    members : Vec<(NodeId,String,String)>,
    tls_config: Option<RSQliteNodeTlsConfig>,
    _no_database_encryption: bool,
) -> anyhow::Result<()>
where
    P: AsRef<Path>,
{
  if http_addr.is_none() {
    return Err(anyhow::anyhow!("http_addr must be sepcified on server initialization"));
  }
  if rpc_addr.is_none() {
    return Err(anyhow::anyhow!("rpc_addr must be sepcified on server initialization"));
  }
  std::fs::create_dir_all(&base_dir)?;
  let http_addr = http_addr.unwrap();
  let rpc_addr = rpc_addr.unwrap();
  let instance_params = InstanceParams {
    http_addr:http_addr.clone(),
    rpc_addr:rpc_addr.clone(),
    tls_config
  };
  
    
  let instance_params_json=serde_json::to_string(&instance_params)?;
  
  tokio::fs::write(base_dir.as_ref().join("instance_params.json"),instance_params_json.as_bytes()).await?;
  
    let (app,handle) = init_rxqlite(node_id,base_dir,instance_params).await?;
    
    
    
    if leader {
      let mut nodes = BTreeMap::new();
      let node = Node {
          api_addr: http_addr,
          rpc_addr: rpc_addr,
          //tls_config:tls_config.clone(),
      };
      nodes.insert(app.id, node);
      app.raft.initialize(nodes).await?;
      
      if members.len() > 0 {
        let mut member_ship:BTreeSet<NodeId> = members.iter().map(|(node_id,_,_)| 
          *node_id).collect();
        
        
        
        
        for (node_id_,api_addr,rpc_addr) in members.into_iter() {
          let node = Node { 
            rpc_addr, 
            api_addr,
            //tls_config: None,
          };
          tracing::debug!("{}({}):adding learner : {}/{}",file!(),line!(),node_id_, node);
          app.raft.add_learner(node_id_, node, true).await?;
          tracing::debug!("{}({}):learner added: {}",file!(),line!(),node_id_);
        }
        /*
        loop {
          tracing::info!("{}({})",file!(),line!());
          let mut cluster_initialized = true;
          let membership_config = {
            let metrics = app.raft.metrics().borrow().clone();
            metrics.membership_config.clone()
          };
          let mut node_iter=membership_config.nodes();
          for node_id in member_ship.iter() {
            if node_iter.position(|(nid,_)| nid==node_id).is_none() {
              cluster_initialized=false;
              break;
            }
          }
          if cluster_initialized {
            break;
          }
          tokio::time::sleep(std::time::Duration::from_millis(250)).await;
          //poll_future(Pin::new(&mut handle), std::time::Duration::from_millis(250)).await;
        }
        */
        tracing::debug!("{}({}):changing membership",file!(),line!());
        member_ship.insert(app.id);
        app.raft.change_membership(member_ship, false).await?;
        
        tracing::debug!("{}({}):membership changed",file!(),line!());
      }
    }
    

    
    
    
    
    
    _ = handle.await;
    Ok(())
    
}


pub async fn start_example_raft_node<P>(
    node_id: NodeId,
    base_dir: P,
    _http_addr: Option<String>,
    _rpc_addr: Option<String>,
    _tls_config: Option<RSQliteNodeTlsConfig>,
    _no_database_encryption: bool,
) -> anyhow::Result<()>
where
    P: AsRef<Path>,
{
    
    let tls_instance_params_json = tokio::fs::read_to_string(base_dir.as_ref().join("instance_params.json")).await?;
    let instance_params: InstanceParams = serde_json::from_str(&tls_instance_params_json)?;
    
    
    let (_,handle) = init_rxqlite(node_id,base_dir,instance_params).await?;
    
    _ = handle.await;
    Ok(())
}

pub use rxqlite_common::{Message,MessageResponse,Value};
/*
#[derive(Debug,Default,Clone,Copy,PartialEq,Eq)]
pub enum Scheme {
  #[default]
  HTTP,
  HTTPS,
}
*/
#[derive(Default,Debug,Clone)]
pub struct ConnectOptions {
  //pub scheme: Scheme,
  pub leader_id: NodeId,
  pub leader_host: String,
  pub leader_port: u16,
  pub tls_config: Option<RSQliteClientTlsConfig>,
}

pub type RXQLiteError = anyhow::Error;

impl ConnectOptions {
  pub async fn connect(&self)->Result<client::RXQLiteClient,RXQLiteError> {
    Ok(client::RXQLiteClient::with_options(self))
  }
}

pub use client::RXQLiteClient as Connection;

pub use rxqlite_common::FromValueRef;

#[cfg(test)]
mod tests;

#[cfg(feature = "test-dependency")]
pub mod tests;