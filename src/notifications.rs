use super::*;

//use tokio::io::{AsyncReadExt, AsyncWriteExt};
//use tokio::net::TcpListener;
use tokio::net::TcpSocket;
use tokio::net::lookup_host;

use tokio_rustls::TlsAcceptor;
//use tokio::net::TcpStream;
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};

use std::sync::Arc;
use tokio_util::bytes::BytesMut;
//use rustls::ServerConfig;
use futures_util::stream::StreamExt;
use futures_util::SinkExt;
use serde_json::{from_slice, to_vec};
use sqlx_sqlite_cipher::notifications::*;
use tokio::io::split;
use tokio::io::{AsyncRead, AsyncWrite};

#[derive(Serialize, Deserialize)]
pub enum NotificationRequest {
    Register,
    Unregister,
}

#[derive(Serialize, Deserialize)]
pub enum NotificationEvent {
    Notification(Notification),
}

async fn server_loop<RW>(stream: RW) -> anyhow::Result<()>
where
    RW: AsyncRead + AsyncWrite + Unpin,
{
    let (reader, writer) = split(stream);
    let mut length_delimited_stream = FramedRead::new(reader, LengthDelimitedCodec::new());
    let mut framed_write = FramedWrite::new(writer, LengthDelimitedCodec::new());

    let mut client_id_receiver: Option<(ClientId, flume::Receiver<Notification>)> = None;

    loop {
        if let Some(client_id_receiver_) = client_id_receiver.as_ref() {
            tokio::select! {
                message = length_delimited_stream.next() => {
                  if let Some(message)=message {
                    let message: NotificationRequest = from_slice(&message
                      .map_err(|err| {
                        NOTIFICATION_DISPATCHER.get().unregister_client(client_id_receiver_.0);
                        err
                      })?
                    ).map_err(|err| {
                        NOTIFICATION_DISPATCHER.get().unregister_client(client_id_receiver_.0);
                        err
                    })?;
                    match message {
                      NotificationRequest::Unregister=> {
                        NOTIFICATION_DISPATCHER.get().unregister_client(client_id_receiver_.0);
                        client_id_receiver=None;
                      }
                      _=>{}
                    }
                  } else {
                    NOTIFICATION_DISPATCHER.get().unregister_client(client_id_receiver_.0);
                    break Ok(());
                  }
                }
                Ok(notification)= client_id_receiver_.1.recv_async() => {
                    tracing::debug!("forwarding notification to client...");
                    let notification_event = NotificationEvent::Notification(notification);

                    let message = to_vec(&notification_event)?;
                    framed_write.send(BytesMut::from(message.as_slice()).freeze()).await?;
                }
            }
        } else {
            let message = length_delimited_stream.next().await;
            if let Some(message) = message {
                let message: NotificationRequest = from_slice(&message?)?;
                match message {
                    NotificationRequest::Register => {
                        tracing::debug!("registering client for notification...");
                        let (client_id, receiver) = NOTIFICATION_DISPATCHER
                            .get_or_init(Default::default)
                            .register_client();
                        client_id_receiver = Some((client_id, receiver));
                    }
                    _ => {}
                }
            } else {
                break Ok(());
            }
        }
    }
}

pub async fn start_notification_server_tls(
    notification_address: String,
    config: tokio_rustls::rustls::ServerConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let acceptor = TlsAcceptor::from(Arc::new(config));
    //let listener = TcpListener::bind(&notification_address).await?;
    
    let socket = TcpSocket::new_v4()?;
    let mut notification_address = lookup_host(&notification_address).await?;
    socket.bind(notification_address.next().unwrap())?;
    if rxqlite_common::IN_TEST.load(rxqlite_common::Ordering::Relaxed) {
      socket.set_reuseaddr(true)?;
    }
    let listener = socket.listen(1024)?;
    
    
    loop {
        let (stream, _) = listener.accept().await?;
        let tls_stream = acceptor.accept(stream).await?;
        tokio::spawn(async move {
            if let Err(e) = server_loop(tls_stream).await {
                tracing::error!("Server loop error: {}", e);
            }
        });
    }
}

pub async fn start_notification_server(
    notification_address: String,
) -> Result<(), Box<dyn std::error::Error>> {
    //let listener = TcpListener::bind(&notification_address).await?;
    let socket = TcpSocket::new_v4()?;
    let mut notification_address = lookup_host(&notification_address).await?;
    socket.bind(notification_address.next().unwrap())?;
    
    if rxqlite_common::IN_TEST.load(rxqlite_common::Ordering::Relaxed) {
      socket.set_reuseaddr(true)?;
    }
    let listener = socket.listen(1024)?;
    loop {
        let (stream, _) = listener.accept().await?;
        tokio::spawn(async move {
            if let Err(e) = server_loop(stream).await {
                tracing::error!("Server loop error: {}", e);
            }
        });
    }
}
