use std::collections::BTreeSet;
use std::sync::Arc;
use std::sync::Mutex;

use openraft::error::NetworkError;
use openraft::error::RPCError;
use openraft::error::RemoteError;
use openraft::RaftMetrics;
use openraft::TryAsRef;
use reqwest::{Client, ClientBuilder};
use serde::de::DeserializeOwned;
//use serde::Deserialize;
use serde::Serialize;

use tokio::time::{timeout, Duration};
//use tokio::io::{AsyncReadExt,AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;
use tokio_util::bytes::BytesMut;
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};

use futures_util::stream::StreamExt;
use futures_util::SinkExt;
use tokio::io::split;
use tokio::io::{ReadHalf, WriteHalf};
use tokio_rustls::rustls::ClientConfig;
use tokio_rustls::rustls::RootCertStore;

use crate::notifications::{NotificationEvent, NotificationRequest};
use serde_json::{from_slice, to_vec};

use crate::sqlite_store::Request;
use crate::typ;
use crate::Node;
use crate::NodeId;

#[derive(Debug)]
struct AllowAnyCertVerifier;

impl tokio_rustls::rustls::client::danger::ServerCertVerifier for AllowAnyCertVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &tokio_rustls::rustls::pki_types::CertificateDer<'_>,
        _intermediates: &[tokio_rustls::rustls::pki_types::CertificateDer<'_>],
        _server_name: &tokio_rustls::rustls::pki_types::ServerName<'_>,
        _ocsp_response: &[u8],
        _now: tokio_rustls::rustls::pki_types::UnixTime,
    ) -> Result<tokio_rustls::rustls::client::danger::ServerCertVerified, tokio_rustls::rustls::Error>
    {
        Ok(tokio_rustls::rustls::client::danger::ServerCertVerified::assertion())
    }
    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &tokio_rustls::rustls::pki_types::CertificateDer<'_>,
        _dss: &tokio_rustls::rustls::DigitallySignedStruct,
    ) -> Result<
        tokio_rustls::rustls::client::danger::HandshakeSignatureValid,
        tokio_rustls::rustls::Error,
    > {
        Ok(tokio_rustls::rustls::client::danger::HandshakeSignatureValid::assertion())
    }
    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &tokio_rustls::rustls::pki_types::CertificateDer<'_>,
        _dss: &tokio_rustls::rustls::DigitallySignedStruct,
    ) -> Result<
        tokio_rustls::rustls::client::danger::HandshakeSignatureValid,
        tokio_rustls::rustls::Error,
    > {
        Ok(tokio_rustls::rustls::client::danger::HandshakeSignatureValid::assertion())
    }
    fn supported_verify_schemes(&self) -> Vec<tokio_rustls::rustls::SignatureScheme> {
        vec![
            tokio_rustls::rustls::SignatureScheme::RSA_PKCS1_SHA1,
            tokio_rustls::rustls::SignatureScheme::ECDSA_SHA1_Legacy,
            tokio_rustls::rustls::SignatureScheme::RSA_PKCS1_SHA256,
            tokio_rustls::rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
            tokio_rustls::rustls::SignatureScheme::RSA_PKCS1_SHA384,
            tokio_rustls::rustls::SignatureScheme::ECDSA_NISTP384_SHA384,
            tokio_rustls::rustls::SignatureScheme::RSA_PKCS1_SHA512,
            tokio_rustls::rustls::SignatureScheme::ECDSA_NISTP521_SHA512,
            tokio_rustls::rustls::SignatureScheme::RSA_PSS_SHA256,
            tokio_rustls::rustls::SignatureScheme::RSA_PSS_SHA384,
            tokio_rustls::rustls::SignatureScheme::RSA_PSS_SHA512,
            tokio_rustls::rustls::SignatureScheme::ED25519,
            tokio_rustls::rustls::SignatureScheme::ED448,
        ]
    }
}

//use crate::TypeConfig;

use rxqlite_common::{Message, MessageResponse, RSQliteClientTlsConfig, Rows, Value};

use crate::ConnectOptions;

pub struct RXQLiteClientBuilder {
    node_id: NodeId,
    node_addr: String,
    //tls_config: Option<RSQliteClientTlsConfig>,
    use_tls: bool,
    accept_invalid_certificates: bool,
}

impl RXQLiteClientBuilder {
    pub fn new(node_id: NodeId, node_addr: String) -> Self {
        Self {
            node_id,
            node_addr,
            //tls_config: None,
            use_tls: false,
            accept_invalid_certificates: false,
        }
    }
    pub fn tls_config(mut self, tls_config: Option<RSQliteClientTlsConfig>) -> Self {
        if let Some(tls_config) = tls_config {
            self.use_tls = true;
            self.accept_invalid_certificates = tls_config.accept_invalid_certificates;
        } else {
            self.use_tls = false;
            self.accept_invalid_certificates = false;
        }
        self
    }
    pub fn use_tls(mut self, use_tls: bool) -> Self {
        self.use_tls = use_tls;
        self
    }
    pub fn accept_invalid_certificates(mut self, accept_invalid_certificates: bool) -> Self {
        self.accept_invalid_certificates = accept_invalid_certificates;
        self
    }
    pub fn build(self) -> RXQLiteClient {
        let mut inner = ClientBuilder::new();
        let use_tls = if self.use_tls {
            if self.accept_invalid_certificates {
                inner = inner.danger_accept_invalid_certs(true);
            }
            true
        } else {
            false
        };
        let inner = inner.build().unwrap();
        RXQLiteClient {
            node: Arc::new(Mutex::new((self.node_id, self.node_addr.clone()))),
            leader: Arc::new(Mutex::new((self.node_id, self.node_addr))),
            inner,
            use_tls,
            notification_stream: None,
            accept_invalid_certificates: self.accept_invalid_certificates,
        }
    }
}

pub enum NetStream {
    Tls(
        FramedWrite<WriteHalf<tokio_rustls::client::TlsStream<TcpStream>>, LengthDelimitedCodec>,
        FramedRead<ReadHalf<tokio_rustls::client::TlsStream<TcpStream>>, LengthDelimitedCodec>,
    ),
    Tcp(
        FramedWrite<WriteHalf<TcpStream>, LengthDelimitedCodec>,
        FramedRead<ReadHalf<TcpStream>, LengthDelimitedCodec>,
    ),
}

impl From<tokio_rustls::client::TlsStream<TcpStream>> for NetStream {
    fn from(stream: tokio_rustls::client::TlsStream<TcpStream>) -> Self {
        let (reader, writer) = split(stream);
        Self::Tls(
            FramedWrite::new(writer, LengthDelimitedCodec::new()),
            FramedRead::new(reader, LengthDelimitedCodec::new()),
        )
    }
}

impl From<TcpStream> for NetStream {
    fn from(stream: TcpStream) -> Self {
        let (reader, writer) = split(stream);
        Self::Tcp(
            FramedWrite::new(writer, LengthDelimitedCodec::new()),
            FramedRead::new(reader, LengthDelimitedCodec::new()),
        )
    }
}

impl NetStream {
    pub async fn write(&mut self, notification_request: NotificationRequest) -> anyhow::Result<()> {
        let message = to_vec(&notification_request)?;
        match self {
            Self::Tls(framed_write, _) => {
                framed_write
                    .send(BytesMut::from(message.as_slice()).freeze())
                    .await?;
            }
            Self::Tcp(framed_write, _) => {
                framed_write
                    .send(BytesMut::from(message.as_slice()).freeze())
                    .await?;
            }
        }
        Ok(())
    }
    pub async fn read(&mut self) -> anyhow::Result<NotificationEvent> {
        match self {
            Self::Tls(_, length_delimited_stream) => {
                let message = length_delimited_stream.next().await;
                if let Some(message) = message {
                    let message: BytesMut = message?;
                    let message: NotificationEvent = from_slice(&message)?;
                    Ok(message)
                } else {
                    Err(anyhow::anyhow!("stream closed"))
                }
            }
            Self::Tcp(_, length_delimited_stream) => {
                let message = length_delimited_stream.next().await;
                if let Some(message) = message {
                    let message: BytesMut = message?;
                    let message: NotificationEvent = from_slice(&message)?;
                    Ok(message)
                } else {
                    Err(anyhow::anyhow!("stream closed"))
                }
            }
        }
    }
    pub async fn read_timeout(
        &mut self,
        timeout_duration: Duration,
    ) -> anyhow::Result<Option<NotificationEvent>> {
        match self {
            Self::Tls(_, length_delimited_stream) => {
                let res = timeout(timeout_duration, length_delimited_stream.next()).await;
                match res {
                    Ok(message) => {
                        if let Some(message) = message {
                            let message: BytesMut = message?;
                            let message: NotificationEvent = from_slice(&message)?;
                            Ok(Some(message))
                        } else {
                            Ok(None)
                        }
                    }
                    Err(_) => Ok(None),
                }
            }
            Self::Tcp(_, length_delimited_stream) => {
                let res = timeout(timeout_duration, length_delimited_stream.next()).await;
                match res {
                    Ok(message) => {
                        if let Some(message) = message {
                            let message: BytesMut = message?;
                            let message: NotificationEvent = from_slice(&message)?;
                            Ok(Some(message))
                        } else {
                            Ok(None)
                        }
                    }
                    Err(_) => Ok(None),
                }
            }
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

    pub accept_invalid_certificates: bool,

    pub notification_stream: Option<NetStream>,
}

impl RXQLiteClient {
    pub fn with_options(options: &ConnectOptions) -> Self {
        let mut inner = ClientBuilder::new();
        let accept_invalid_certificates = if let Some(tls_config) = options.tls_config.as_ref() {
            inner = inner.use_rustls_tls();
            if tls_config.accept_invalid_certificates {
                inner = inner.danger_accept_invalid_certs(true);
                true
            } else {
                false
            }
        } else {
            false
        };
        let inner = inner.build().unwrap();
        let node = Arc::new(Mutex::new((
            options.leader_id,
            format!("{}:{}", options.leader_host, options.leader_port),
        )));
        let leader = Arc::new(Mutex::new((
            options.leader_id,
            format!("{}:{}", options.leader_host, options.leader_port),
        )));
        Self {
            node,
            leader,
            inner,
            use_tls: options.tls_config.is_some(),
            notification_stream: None,
            accept_invalid_certificates,
        }
    }

    /// Create a client with a leader node id and a node manager to get node address by node id.
    pub fn new(node_id: NodeId, node_addr: String) -> Self {
        Self {
            node: Arc::new(Mutex::new((node_id, node_addr.clone()))),
            leader: Arc::new(Mutex::new((node_id, node_addr))),
            inner: Client::new(),
            use_tls: false,
            notification_stream: None,
            accept_invalid_certificates: false,
        }
    }

    // --- Application API

    /// Submit a write request to the raft cluster.
    ///
    /// The request will be processed by raft protocol: it will be replicated to a quorum and then
    /// will be applied to state machine.
    ///
    /// The result of applying the request will be returned.
    pub async fn sql(
        &self,
        req: &Request,
    ) -> Result<typ::ClientWriteResponse, typ::RPCError<typ::ClientWriteError>> {
        self.send_rpc_to_leader("api/sql", Some(req)).await
    }
    
    pub async fn sql_with_retries_and_delay(
        &self,
        req: &Request,
        mut retries: usize,
        delay_between_retries: Duration,
    ) -> Result<typ::ClientWriteResponse, typ::RPCError<typ::ClientWriteError>> {
        retries += 1;
        loop {
          match self.send_rpc_to_leader("api/sql", Some(req)).await {
            Ok(res)=>return Ok(res),
            Err(rpc_err)=> {
              if let RPCError::RemoteError(remote_err) = &rpc_err {
                let raft_err: &typ::RaftError<_> = &remote_err.source;

                if let Some(typ::ForwardToLeader {
                    leader_id,   //: Some(leader_id),
                    leader_node : _, //: Some(leader_node),
                    ..
                }) = raft_err.forward_to_leader() {
                  if leader_id.is_some() {
                    return Err(rpc_err);
                  } else {
                    retries-=1;
                    if retries == 0 {
                      return Err(rpc_err);
                    }
                    tokio::time::sleep(delay_between_retries).await;
                  }
                } else {
                  return Err(rpc_err);
                }
              } else {
                return Err(rpc_err);
              }
            }
          }
        }
    }
    
    pub async fn consistent_sql(
        &self,
        req: &Request,
    ) -> Result<typ::ClientWriteResponse, typ::RPCError<typ::ClientWriteError>> {
        self.send_rpc_to_leader("api/sql-consistent", Some(req))
            .await
    }
    pub async fn execute(
        &self,
        query: &str,
        arguments: Vec<Value>,
    ) -> Result<Rows, crate::RXQLiteError> {
        let req = Message::Execute(query.into(), arguments);
        let res: Result<typ::ClientWriteResponse, typ::RPCError<typ::ClientWriteError>> = self
            .send_rpc_to_leader("api/sql-consistent", Some(&req))
            .await;
        match res {
            Ok(res) => match res.data {
                Some(res) => match res {
                    MessageResponse::Rows(rows) => Ok(rows),
                    MessageResponse::Error(error) => Err(anyhow::anyhow!(error)),
                },
                _ => Ok(Rows::default()),
            },
            Err(err) => Err(anyhow::anyhow!(err)),
        }
    }
    pub async fn fetch_all(
        &self,
        query: &str,
        arguments: Vec<Value>,
    ) -> Result<Rows, crate::RXQLiteError> {
        let req = Message::Fetch(query.into(), arguments);
        let res: Result<typ::ClientWriteResponse, typ::RPCError<typ::ClientWriteError>> = self
            .send_rpc_to_leader("api/sql-consistent", Some(&req))
            .await;
        match res {
            Ok(res) => match res.data {
                Some(res) => match res {
                    MessageResponse::Rows(rows) => Ok(rows),
                    MessageResponse::Error(error) => Err(anyhow::anyhow!(error)),
                },
                _ => Ok(Rows::default()),
            },
            Err(err) => Err(anyhow::anyhow!(err)),
        }
    }
    pub async fn fetch_one(
        &self,
        query: &str,
        arguments: Vec<Value>,
    ) -> Result<rxqlite_common::Row, crate::RXQLiteError> {
        let req = Message::FetchOne(query.into(), arguments);
        let res: Result<typ::ClientWriteResponse, typ::RPCError<typ::ClientWriteError>> = self
            .send_rpc_to_leader("api/sql-consistent", Some(&req))
            .await;
        match res {
            Ok(res) => match res.data {
                Some(res) => match res {
                    MessageResponse::Rows(mut rows) => {
                        if rows.len() >= 1 {
                            Ok(rows.remove(0))
                        } else {
                            Err(anyhow::anyhow!("no row matching query"))
                        }
                    }
                    MessageResponse::Error(error) => Err(anyhow::anyhow!(error)),
                },
                _ => Err(anyhow::anyhow!("no row matching query")),
            },
            Err(err) => Err(anyhow::anyhow!(err)),
        }
    }
    pub async fn fetch_optional(
        &self,
        query: &str,
        arguments: Vec<Value>,
    ) -> Result<Option<rxqlite_common::Row>, crate::RXQLiteError> {
        let req = Message::FetchOptional(query.into(), arguments);
        let res: Result<typ::ClientWriteResponse, typ::RPCError<typ::ClientWriteError>> = self
            .send_rpc_to_leader("api/sql-consistent", Some(&req))
            .await;
        match res {
            Ok(res) => match res.data {
                Some(res) => match res {
                    MessageResponse::Rows(mut rows) => {
                        if rows.len() >= 1 {
                            Ok(Some(rows.remove(0)))
                        } else {
                            Ok(None)
                        }
                    }
                    MessageResponse::Error(error) => Err(anyhow::anyhow!(error)),
                },
                _ => Ok(None),
            },
            Err(err) => Err(anyhow::anyhow!(err)),
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
        self.send_rpc_to_leader("cluster/add-learner", Some(&req))
            .await
    }

    /// Change membership to the specified set of nodes.
    ///
    /// All nodes in `req` have to be already added as learner with [`add_learner`],
    /// or an error [`LearnerNotFound`] will be returned.
    pub async fn change_membership(
        &self,
        req: &BTreeSet<NodeId>,
    ) -> Result<typ::ClientWriteResponse, typ::RPCError<typ::ClientWriteError>> {
        self.send_rpc_to_leader("cluster/change-membership", Some(req))
            .await
    }

    /// Get the metrics about the cluster.
    ///
    /// Metrics contains various information about the cluster, such as current leader,
    /// membership config, replication status etc.
    /// See [`RaftMetrics`].
    pub async fn metrics(&self) -> Result<RaftMetrics<NodeId, Node>, typ::RPCError> {
        self.do_send_rpc_to_leader("cluster/metrics", None::<&()>)
            .await
    }

    /// Get the metrics about the cluster from the original node.
    ///
    /// Metrics contains various information about the cluster, such as current leader,
    /// membership config, replication status etc.
    /// See [`RaftMetrics`].
    pub async fn node_metrics(&self) -> Result<RaftMetrics<NodeId, Node>, typ::RPCError> {
        self.do_send_rpc_to_node(&self.node, "cluster/metrics", None::<&()>)
            .await
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
            (
                t.0,
                format!(
                    "{}://{}/{}",
                    if self.use_tls { "https" } else { "http" },
                    target_addr,
                    uri
                ),
            )
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

        let res: Result<Resp, Err> = resp
            .json()
            .await
            .map_err(|e| RPCError::Network(NetworkError::new(&e)))?;
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
        self.do_send_rpc_to_node(&self.leader, uri, req).await
    }

    /// Try the best to send a request to the leader.
    ///
    /// If the target node is not a leader, a `ForwardToLeader` error will be
    /// returned and this client will retry at most 3 times to contact the updated leader.
    async fn send_rpc_to_leader<Req, Resp, Err>(
        &self,
        uri: &str,
        req: Option<&Req>,
    ) -> Result<Resp, typ::RPCError<Err>>
    where
        Req: Serialize + 'static,
        Resp: Serialize + DeserializeOwned,
        Err: std::error::Error
            + Serialize
            + DeserializeOwned
            + TryAsRef<typ::ForwardToLeader>
            + Clone,
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
                    leader_id,   //: Some(leader_id),
                    leader_node, //: Some(leader_node),
                    ..
                }) = raft_err.forward_to_leader()
                {
                    // Update target to the new leader.
                    if let (Some(leader_id), Some(leader_node)) = (leader_id, leader_node) {
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

impl RXQLiteClient {
    pub async fn stop_listening_for_notifications(&mut self) -> anyhow::Result<()> {
        if self.notification_stream.is_none() {
            return Ok(());
        }
        self.notification_stream
            .as_mut()
            .unwrap()
            .write(NotificationRequest::Unregister)
            .await?;
        self.notification_stream.take();
        Ok(())
    }
    pub async fn start_listening_for_notifications(
        &mut self,
        notifications_addr: &str,
    ) -> anyhow::Result<()> {
        if self.notification_stream.is_some() {
            return Ok(());
        }
        if self.use_tls {
            let root_certs = RootCertStore::empty();
            let mut config/*: rustls::ConfigBuilder<ClientConfig,rustls::WantsVersions>*/= ClientConfig::builder()
        .with_root_certificates(root_certs)
        .with_no_client_auth();
            if self.accept_invalid_certificates {
                config
                    .dangerous()
                    .set_certificate_verifier(Arc::new(AllowAnyCertVerifier));
            }

            let connector = TlsConnector::from(Arc::new(config));
            let server_name = rustls::pki_types::ServerName::try_from(
                notifications_addr.split(":").next().unwrap(),
            )?;
            let stream = TcpStream::connect(notifications_addr).await?;
            let tls_stream = connector.connect(server_name.to_owned(), stream).await?;
            let mut notification_stream = NetStream::from(tls_stream);
            notification_stream
                .write(NotificationRequest::Register)
                .await?;
            self.notification_stream = Some(notification_stream);
            Ok(())
        } else {
            let stream = TcpStream::connect(notifications_addr).await?;
            let mut notification_stream = NetStream::from(stream);
            notification_stream
                .write(NotificationRequest::Register)
                .await?;
            self.notification_stream = Some(notification_stream);
            Ok(())
        }
    }
}
