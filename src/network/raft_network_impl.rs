use std::any::Any;
use std::fmt::Display;

use openraft::error::InstallSnapshotError;
use openraft::error::NetworkError;
use openraft::error::RPCError;
use openraft::error::RaftError;
use openraft::error::RemoteError;
use openraft::network::RPCOption;
use openraft::network::RaftNetwork;
use openraft::network::RaftNetworkFactory;
use openraft::raft::AppendEntriesRequest;
use openraft::raft::AppendEntriesResponse;
use openraft::raft::InstallSnapshotRequest;
use openraft::raft::InstallSnapshotResponse;
use openraft::raft::VoteRequest;
use openraft::raft::VoteResponse;
use openraft::AnyError;
use serde::de::DeserializeOwned;
//use toy_rpc_ha421::pubsub::AckModeNone;
use toy_rpc_ha421::Client;

use super::raft::RaftClientStub;
use crate::Node;
use crate::NodeId;
use crate::TypeConfig;

use rustls::{pki_types::CertificateDer, ClientConfig, RootCertStore};
use rustls::client::danger::ServerCertVerified;
use rustls::client::danger::ServerCertVerifier;

use crate::RSQliteNodeTlsConfig;
use std::sync::Arc;
use std::net::{IpAddr, ToSocketAddrs};

#[derive(Debug)]
struct AllowAnyCertVerifier;

impl ServerCertVerifier for AllowAnyCertVerifier {
    /// Will verify the certificate is valid in the following ways:
    /// - Signed by a  trusted `RootCertStore` CA
    /// - Not Expired
    /// - Valid for DNS entry
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &rustls::pki_types::ServerName<'_>,
        _ocsp_response: &[u8],
        _now: rustls::pki_types::UnixTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }
    fn verify_tls12_signature(
      &self,
      _message: &[u8],
      _cert: &CertificateDer<'_>,
      _dss: &rustls::DigitallySignedStruct
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
      Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }
    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
      Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }
    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
      vec![
        rustls::SignatureScheme::RSA_PKCS1_SHA1,
        rustls::SignatureScheme::ECDSA_SHA1_Legacy,
        rustls::SignatureScheme::RSA_PKCS1_SHA256,
        rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
        rustls::SignatureScheme::RSA_PKCS1_SHA384,
        rustls::SignatureScheme::ECDSA_NISTP384_SHA384,
        rustls::SignatureScheme::RSA_PKCS1_SHA512,
        rustls::SignatureScheme::ECDSA_NISTP521_SHA512,
        rustls::SignatureScheme::RSA_PSS_SHA256,
        rustls::SignatureScheme::RSA_PSS_SHA384,
        rustls::SignatureScheme::RSA_PSS_SHA512,
        rustls::SignatureScheme::ED25519,
        rustls::SignatureScheme::ED448,
      ]
    }
}


pub struct Network {
  pub tls_config : Option<RSQliteNodeTlsConfig>,
}

// NOTE: This could be implemented also on `Arc<ExampleNetwork>`, but since it's empty, implemented
// directly.
impl RaftNetworkFactory<TypeConfig> for Network {
    type Network = NetworkConnection;

    #[tracing::instrument(level = "debug", skip_all)]
    async fn new_client(&mut self, target: NodeId, node: &Node) -> Self::Network {
        if let Some(tls_config)=self.tls_config.as_ref() {
          //let addr = format!("{}", node.rpc_addr);
          let addr = node.rpc_addr.clone();
          
          let parts: Vec<&str> = addr.split(':').collect();
          let host = parts[0];
          let port: u16 = parts[1].parse().unwrap();

          let (addr,domain) = match host.parse::<IpAddr>() {
            Ok(_) => {
              (host.to_string(),host.to_string())
            }
            Err(_) => {
              match (host, port).to_socket_addrs() {
                Ok(mut addrs) => {
                  match addrs.next() {
                    Some(addr) => {
                      (addr.to_string(),host.to_string())
                    }
                    None => {
                      tracing::error!("No address found for {}",host);
                      (host.to_string(),host.to_string())
                    }
                  }
                }
                Err(e) => {
                  tracing::error!("DNS resolution error for {}({})",host,e);
                  (host.to_string(),host.to_string())
                }
              }
            }
          };
          
          let addr = format!("{}:{}", addr,port);
          
          if tls_config.accept_invalid_certificates {
            let root_certs = RootCertStore::empty();
            let mut config/*: rustls::ConfigBuilder<ClientConfig,rustls::WantsVersions>*/= ClientConfig::builder()
              //.with_safe_default_cipher_suites()
              //.with_safe_default_kx_groups()
              //.with_safe_default_protocol_versions()
              .with_root_certificates(root_certs)
              .with_no_client_auth();
            config.dangerous().set_certificate_verifier(Arc::new(AllowAnyCertVerifier));
            //let domain = addr.clone();
            
            let client = Client::dial_with_tls_config(&addr,&domain,config).await.ok();
            tracing::debug!("new_client: is_none: {}", client.is_none());

            NetworkConnection { addr, domain: domain.to_string() , client, target, tls_config: self.tls_config.clone() }
          } else {
            let root_certs = RootCertStore::empty();
            let config = ClientConfig::builder()
              //.with_safe_defaults()
              .with_root_certificates(root_certs)
              .with_no_client_auth();
            
            
            let client = Client::dial_with_tls_config(&addr,&domain,config).await.ok();
            tracing::debug!("new_client: is_none: {}", client.is_none());

            NetworkConnection { addr, domain: domain.to_string(), client, target, tls_config: self.tls_config.clone() }
          }
          
        } else {
          let addr = format!("ws://{}", node.rpc_addr);

          let client = Client::dial_websocket(&addr).await.ok();
          tracing::debug!("new_client: is_none: {}", client.is_none());

          NetworkConnection { addr, client, target, domain : String::default(), tls_config: self.tls_config.clone() }
        }
    }
}

pub struct NetworkConnection {
    addr: String,
    domain: String,
    client: Option<Client/*<AckModeNone>*/>,
    target: NodeId,
    tls_config : Option<RSQliteNodeTlsConfig>,
}
impl NetworkConnection {
    async fn c<E: std::error::Error + DeserializeOwned>(
        &mut self,
    ) -> Result<&Client/*<AckModeNone>*/, RPCError<NodeId, Node, E>> {
        if self.client.is_none() {
            if let Some(tls_config) =  self.tls_config.as_ref() {
              
              if tls_config.accept_invalid_certificates {
               let root_certs = RootCertStore::empty();
               let mut config = ClientConfig::builder()
                //.with_safe_default_cipher_suites()
                //.with_safe_default_kx_groups()
                //.with_safe_default_protocol_versions()
                //.unwrap()
                .with_root_certificates(root_certs)
                .with_no_client_auth()
              ;
                config.dangerous().set_certificate_verifier(
                  Arc::new(AllowAnyCertVerifier)
                );
                //let domain = self.addr.clone();
                
                self.client = Client::dial_with_tls_config(&self.addr,&self.domain,config).await.ok();
                

                
              } else {
                let root_certs = RootCertStore::empty();
                let config = ClientConfig::builder()
                  //.with_safe_defaults()
                  .with_root_certificates(root_certs)
                  .with_no_client_auth();
                
                
                self.client = Client::dial_with_tls_config(&self.addr,&self.domain,config).await.ok();
                
              }
            } else {
              self.client = Client::dial_websocket(&self.addr).await.ok();
            }
        }
        self.client.as_ref().ok_or_else(|| RPCError::Network(NetworkError::from(AnyError::default())))
    }
}

#[derive(Debug)]
struct ErrWrap(Box<dyn std::error::Error>);

impl Display for ErrWrap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::error::Error for ErrWrap {}

fn to_error<E: std::error::Error + 'static + Clone>(e: toy_rpc_ha421::Error, target: NodeId) -> RPCError<NodeId, Node, E> {
    match e {
        toy_rpc_ha421::Error::IoError(e) => RPCError::Network(NetworkError::new(&e)),
        toy_rpc_ha421::Error::ParseError(e) => RPCError::Network(NetworkError::new(&ErrWrap(e))),
        toy_rpc_ha421::Error::Internal(e) => {
            let any: &dyn Any = &e;
            let error: &E = any.downcast_ref().unwrap();
            RPCError::RemoteError(RemoteError::new(target, error.clone()))
        }
        e @ (toy_rpc_ha421::Error::InvalidArgument
        | toy_rpc_ha421::Error::ServiceNotFound
        | toy_rpc_ha421::Error::MethodNotFound
        | toy_rpc_ha421::Error::ExecutionError(_)
        | toy_rpc_ha421::Error::Canceled(_)
        | toy_rpc_ha421::Error::Timeout(_)
        | toy_rpc_ha421::Error::MaxRetriesReached(_)) => RPCError::Network(NetworkError::new(&e)),
    }
}

// With nightly-2023-12-20, and `err(Debug)` in the instrument macro, this gives the following lint
// warning. Without `err(Debug)` it is OK. Suppress it with `#[allow(clippy::blocks_in_conditions)]`
//
// warning: in a `match` scrutinee, avoid complex blocks or closures with blocks; instead, move the
// block or closure higher and bind it with a `let`
//
//    --> src/network/raft_network_impl.rs:99:91
//     |
// 99  |       ) -> Result<AppendEntriesResponse<NodeId>, RPCError<NodeId, Node, RaftError<NodeId>>>
// {
//     |  ___________________________________________________________________________________________^
// 100 | |         tracing::debug!(req = debug(&req), "append_entries");
// 101 | |
// 102 | |         let c = self.c().await?;
// ...   |
// 108 | |         raft.append(req).await.map_err(|e| to_error(e, self.target))
// 109 | |     }
//     | |_____^
//     |
//     = help: for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#blocks_in_conditions
//     = note: `#[warn(clippy::blocks_in_conditions)]` on by default
#[allow(clippy::blocks_in_conditions)]
impl RaftNetwork<TypeConfig> for NetworkConnection {
    #[tracing::instrument(level = "debug", skip_all, err(Debug))]
    async fn append_entries(
        &mut self,
        req: AppendEntriesRequest<TypeConfig>,
        _option: RPCOption,
    ) -> Result<AppendEntriesResponse<NodeId>, RPCError<NodeId, Node, RaftError<NodeId>>> {
        tracing::debug!(req = debug(&req), "append_entries");

        let c = self.c().await?;
        tracing::debug!("got connection");

        let raft = c.raft();
        tracing::debug!("got raft");

        raft.append(req).await.map_err(|e| to_error(e, self.target))
    }

    #[tracing::instrument(level = "debug", skip_all, err(Debug))]
    async fn install_snapshot(
        &mut self,
        req: InstallSnapshotRequest<TypeConfig>,
        _option: RPCOption,
    ) -> Result<InstallSnapshotResponse<NodeId>, RPCError<NodeId, Node, RaftError<NodeId, InstallSnapshotError>>> {
        tracing::debug!(req = debug(&req), "install_snapshot");
        self.c().await?.raft().snapshot(req).await.map_err(|e| to_error(e, self.target))
    }

    #[tracing::instrument(level = "debug", skip_all, err(Debug))]
    async fn vote(
        &mut self,
        req: VoteRequest<NodeId>,
        _option: RPCOption,
    ) -> Result<VoteResponse<NodeId>, RPCError<NodeId, Node, RaftError<NodeId>>> {
        tracing::debug!(req = debug(&req), "vote");
        self.c().await?.raft().vote(req).await.map_err(|e| to_error(e, self.target))
    }
}
