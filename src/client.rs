use std::collections::BTreeSet;
use std::sync::Arc;
use std::sync::Mutex;

use openraft::error::NetworkError;
use openraft::error::RPCError;
use openraft::error::RemoteError;
use openraft::RaftMetrics;
use openraft::TryAsRef;
use reqwest::{Client,ClientBuilder};
use serde::de::DeserializeOwned;
//use serde::Deserialize;
use serde::Serialize;

use crate::typ;
use crate::Node;
use crate::NodeId;
use crate::sqlite_store::Request;
//use crate::TypeConfig;

use rxqlite_common::{
  Message,
  MessageResponse,
  Value,
  Rows,
  RSQliteClientTlsConfig,
};

 
use crate::ConnectOptions;

pub struct RXQLiteClientBuilder {
  node_id: NodeId,
  node_addr: String,
  //tls_config: Option<RSQliteClientTlsConfig>,
  use_tls: bool,
  accept_invalid_certificates: bool,
}

impl RXQLiteClientBuilder {
  pub fn new(node_id: NodeId, node_addr: String)->Self {
    
    Self {
      node_id,
      node_addr,
      //tls_config: None,
      use_tls: false,
      accept_invalid_certificates:false,
    }
  }
  pub fn tls_config(mut self,tls_config: Option<RSQliteClientTlsConfig>)->Self {
    if let Some(tls_config) = tls_config {
      self.use_tls=true;
      self.accept_invalid_certificates = tls_config.accept_invalid_certificates;
    } else {
      self.use_tls=false;
      self.accept_invalid_certificates = false;
    }
    self
  }
  pub fn use_tls(mut self,use_tls: bool)->Self {
    self.use_tls = use_tls;
    self
  }
  pub fn accept_invalid_certificates(mut self,accept_invalid_certificates: bool)->Self {
    self.accept_invalid_certificates = accept_invalid_certificates;
    self
  }
  pub fn build(self)->RXQLiteClient {
    let mut inner = ClientBuilder::new();
    let use_tls = if self.use_tls {
      if self.accept_invalid_certificates {
        inner = inner.danger_accept_invalid_certs(true);
      }
      true
    } else {
      false
    };
    let inner=inner.build().unwrap();
    RXQLiteClient {
        node: Arc::new(Mutex::new((self.node_id, self.node_addr.clone()))),
        leader: Arc::new(Mutex::new((self.node_id, self.node_addr))),
        inner,
        use_tls,
    }
  }
}
 

pub struct RXQLiteClient {
    /// The leader node to send request to.
    ///
    /// All traffic should be sent to the leader in a cluster.
    pub leader: Arc<Mutex<(NodeId, String)>>,
    
    /// The original node to send request to.
    ///
    /// Mainly used to get node metrics.
    pub node: Arc<Mutex<(NodeId, String)>>,

    pub inner: Client,
    
    pub use_tls: bool,
    
}

impl RXQLiteClient {

    pub fn with_options(options: &ConnectOptions) -> Self {
        let mut inner = ClientBuilder::new();
        if let Some(tls_config) = options.tls_config.as_ref() {
          inner = inner.use_rustls_tls();
          if tls_config.accept_invalid_certificates {
            inner = inner.danger_accept_invalid_certs(true);
          }
          
        }
        let inner=inner.build().unwrap();
        let node = Arc::new(Mutex::new((options.leader_id, format!("{}:{}",options.leader_host,options.leader_port))));
        let leader = Arc::new(Mutex::new((options.leader_id, format!("{}:{}",options.leader_host,options.leader_port))));
        Self {
            node,
            leader,
            inner,
            use_tls: options.tls_config.is_some(),
        }
    }


    /// Create a client with a leader node id and a node manager to get node address by node id.
    pub fn new(node_id: NodeId, node_addr: String) -> Self {
        Self {
            node: Arc::new(Mutex::new((node_id, node_addr.clone()))),
            leader: Arc::new(Mutex::new((node_id, node_addr))),
            inner: Client::new(),
            use_tls: false,
        }
    }

    // --- Application API

    /// Submit a write request to the raft cluster.
    ///
    /// The request will be processed by raft protocol: it will be replicated to a quorum and then
    /// will be applied to state machine.
    ///
    /// The result of applying the request will be returned.
    pub async fn sql(&self, req: &Request) -> Result<typ::ClientWriteResponse, typ::RPCError<typ::ClientWriteError>> {
        self.send_rpc_to_leader("api/sql", Some(req)).await
    }
    pub async fn consistent_sql(&self, req: &Request) -> Result<typ::ClientWriteResponse, typ::RPCError<typ::ClientWriteError>> {
        self.send_rpc_to_leader("api/sql-consistent", Some(req)).await
    }
    pub async fn execute(&self, query: &str,arguments: Vec<Value>) -> Result<Rows, crate::RXQLiteError> {
        let req = Message::Execute(query.into(),arguments);
        let res: Result<typ::ClientWriteResponse, typ::RPCError<typ::ClientWriteError>>
          =self.send_rpc_to_leader("api/sql-consistent", Some(&req)).await;
        match res {
          Ok(res)=>{
            match res.data {
              Some(res)=>{
                match res {
                  MessageResponse::Rows(rows)=>{
                    Ok(rows)
                  }
                  MessageResponse::Error(error)=>{
                    Err(anyhow::anyhow!(error))
                  }
                }
              }
              _=> {
                Ok(Rows::default())
              }
            }
          }
          Err(err)=>{
            Err(anyhow::anyhow!(err))
          }
        }
    }
    pub async fn fetch_all(&self, query: &str,arguments: Vec<Value>) -> Result<Rows, crate::RXQLiteError> {
        let req = Message::Fetch(query.into(),arguments);
        let res: Result<typ::ClientWriteResponse, typ::RPCError<typ::ClientWriteError>>
          =self.send_rpc_to_leader("api/sql-consistent", Some(&req)).await;
        match res {
          Ok(res)=>{
            match res.data {
              Some(res)=>{
                match res {
                  MessageResponse::Rows(rows)=>{
                    Ok(rows)
                  }
                  MessageResponse::Error(error)=>{
                    Err(anyhow::anyhow!(error))
                  }
                }
              }
              _=> {
                Ok(Rows::default())
              }
            }
          }
          Err(err)=>{
            Err(anyhow::anyhow!(err))
          }
        }
    }
    pub async fn fetch_one(&self, query: &str,arguments: Vec<Value>) -> Result<rxqlite_common::Row, crate::RXQLiteError> {
        let req = Message::FetchOne(query.into(),arguments);
        let res: Result<typ::ClientWriteResponse, typ::RPCError<typ::ClientWriteError>>
          =self.send_rpc_to_leader("api/sql-consistent", Some(&req)).await;
        match res {
          Ok(res)=>{
            match res.data {
              Some(res)=>{
                match res {
                  MessageResponse::Rows(mut rows)=>{
                    if rows.len() >= 1 {
                      Ok(rows.remove(0))
                    } else {
                      Err(anyhow::anyhow!("no row matching query"))
                    }
                  }
                  MessageResponse::Error(error)=>{
                    Err(anyhow::anyhow!(error))
                  }
                }
              }
              _=> {
                Err(anyhow::anyhow!("no row matching query"))
              }
            }
          }
          Err(err)=>{
            Err(anyhow::anyhow!(err))
          }
        }
    }
    pub async fn fetch_optional(&self, query: &str,arguments: Vec<Value>) -> Result<Option<rxqlite_common::Row>, crate::RXQLiteError> {
        let req = Message::FetchOptional(query.into(),arguments);
        let res: Result<typ::ClientWriteResponse, typ::RPCError<typ::ClientWriteError>>
          =self.send_rpc_to_leader("api/sql-consistent", Some(&req)).await;
        match res {
          Ok(res)=>{
            match res.data {
              Some(res)=>{
                match res {
                  MessageResponse::Rows(mut rows)=>{
                    if rows.len() >= 1 {
                      Ok(Some(rows.remove(0)))
                    } else {
                      Ok(None)
                    }
                  }
                  MessageResponse::Error(error)=>{
                    Err(anyhow::anyhow!(error))
                  }
                }
              }
              _=> {
                Ok(None)
              }
            }
          }
          Err(err)=>{
            Err(anyhow::anyhow!(err))
          }
        }
    }
    
    
    // --- Cluster management API

    /// Add a node as learner.
    ///
    /// The node to add has to exist, i.e., being added with `write(ExampleRequest::AddNode{})`
    pub async fn add_learner(
        &self,
        req: (NodeId, String, String),
    ) -> Result<typ::ClientWriteResponse, typ::RPCError<typ::ClientWriteError>> {
        self.send_rpc_to_leader("cluster/add-learner", Some(&req)).await
    }

    /// Change membership to the specified set of nodes.
    ///
    /// All nodes in `req` have to be already added as learner with [`add_learner`],
    /// or an error [`LearnerNotFound`] will be returned.
    pub async fn change_membership(
        &self,
        req: &BTreeSet<NodeId>,
    ) -> Result<typ::ClientWriteResponse, typ::RPCError<typ::ClientWriteError>> {
        self.send_rpc_to_leader("cluster/change-membership", Some(req)).await
    }

    /// Get the metrics about the cluster.
    ///
    /// Metrics contains various information about the cluster, such as current leader,
    /// membership config, replication status etc.
    /// See [`RaftMetrics`].
    pub async fn metrics(&self) -> Result<RaftMetrics<NodeId, Node>, typ::RPCError> {
        self.do_send_rpc_to_leader("cluster/metrics", None::<&()>).await
    }
    
    /// Get the metrics about the cluster from the original node.
    ///
    /// Metrics contains various information about the cluster, such as current leader,
    /// membership config, replication status etc.
    /// See [`RaftMetrics`].
    pub async fn node_metrics(&self) -> Result<RaftMetrics<NodeId, Node>, typ::RPCError> {
        self.do_send_rpc_to_node(&self.node,"cluster/metrics", None::<&()>).await
    }
    
    // --- Internal methods

    /// Send RPC to specified node.
    ///
    /// It sends out a POST request if `req` is Some. Otherwise a GET request.
    /// The remote endpoint must respond a reply in form of `Result<T, E>`.
    /// An `Err` happened on remote will be wrapped in an [`RPCError::RemoteError`].
    async fn do_send_rpc_to_node<Req, Resp, Err>(
        &self,
        dest_node: &Arc<Mutex<(NodeId, String)>>,
        uri: &str,
        req: Option<&Req>,
    ) -> Result<Resp, RPCError<NodeId, Node, Err>>
    where
        Req: Serialize + 'static,
        Resp: Serialize + DeserializeOwned,
        Err: std::error::Error + Serialize + DeserializeOwned,
    {
        let (node_id, url) = {
            let t = dest_node.lock().unwrap();
            let target_addr = &t.1;
        (t.0, format!("{}://{}/{}", if self.use_tls {"https" } else { "http" },target_addr, uri))
        };

        let resp = if let Some(r) = req {
            println!(
                ">>> client send request to {}: {}",
                url,
                serde_json::to_string_pretty(&r).unwrap()
            );
            self.inner.post(url.clone()).json(r)
        } else {
            println!(">>> client send request to {}", url,);
            self.inner.get(url.clone())
        }
        .send()
        .await
        .map_err(|e| RPCError::Network(NetworkError::new(&e)))?;

        let res: Result<Resp, Err> = resp.json().await.map_err(|e| RPCError::Network(NetworkError::new(&e)))?;
        println!(
            "<<< client recv reply from {}: {}",
            url,
            serde_json::to_string_pretty(&res).unwrap()
        );

        res.map_err(|e| RPCError::RemoteError(RemoteError::new(node_id, e)))
    }
    /// Send RPC to specified node.
    ///
    /// It sends out a POST request if `req` is Some. Otherwise a GET request.
    /// The remote endpoint must respond a reply in form of `Result<T, E>`.
    /// An `Err` happened on remote will be wrapped in an [`RPCError::RemoteError`].
    
    
    async fn do_send_rpc_to_leader<Req, Resp, Err>(
        &self,
        uri: &str,
        req: Option<&Req>,
    ) -> Result<Resp, RPCError<NodeId, Node, Err>>
    where
        Req: Serialize + 'static,
        Resp: Serialize + DeserializeOwned,
        Err: std::error::Error + Serialize + DeserializeOwned,
    {
        self.do_send_rpc_to_node(&self.leader,uri,req).await
    }

    /// Try the best to send a request to the leader.
    ///
    /// If the target node is not a leader, a `ForwardToLeader` error will be
    /// returned and this client will retry at most 3 times to contact the updated leader.
    async fn send_rpc_to_leader<Req, Resp, Err>(&self, uri: &str, req: Option<&Req>) -> Result<Resp, typ::RPCError<Err>>
    where
        Req: Serialize + 'static,
        Resp: Serialize + DeserializeOwned,
        Err: std::error::Error + Serialize + DeserializeOwned + TryAsRef<typ::ForwardToLeader> + Clone,
    {
        // Retry at most 3 times to find a valid leader.
        let mut n_retry = 3;

        loop {
            let res: Result<Resp, typ::RPCError<Err>> = self.do_send_rpc_to_leader(uri, req).await;

            let rpc_err = match res {
                Ok(x) => return Ok(x),
                Err(rpc_err) => rpc_err,
            };

            if let RPCError::RemoteError(remote_err) = &rpc_err {
                let raft_err: &typ::RaftError<_> = &remote_err.source;

                if let Some(typ::ForwardToLeader {
                    leader_id,//: Some(leader_id),
                    leader_node,//: Some(leader_node),
                    ..
                }) = raft_err.forward_to_leader()
                {
                    // Update target to the new leader.
                    if let (Some(leader_id),Some(leader_node)) = (leader_id,leader_node)
                    {
                        let mut t = self.leader.lock().unwrap();
                        let api_addr = leader_node.api_addr.clone();
                        *t = (*leader_id, api_addr);
                    } else {
                      break Err(rpc_err);
                    }

                    n_retry -= 1;
                    if n_retry > 0 {
                        continue;
                    }
                }
            }

            return Err(rpc_err);
        }
    }
}
