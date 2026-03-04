use std::{error::Error, sync::Arc};

use tokio::sync::watch;

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use crate::{ReliableClientMessage, MessageSize, PlayerId, ReliableServerMessage};
use quinn::crypto::rustls::QuicClientConfig;
use quinn::{
    ClientConfig, Connection, Endpoint,
    rustls::{self},
};
use rkyv::rancor;
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};

async fn connect_to_server(
    server_addr: SocketAddr,
) -> Result<(Endpoint, Connection), Box<dyn Error + Send + Sync + 'static>> {
    let mut endpoint = Endpoint::client(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0))?;

    endpoint.set_default_client_config(ClientConfig::new(Arc::new(QuicClientConfig::try_from(
        rustls::ClientConfig::builder()
            .dangerous()
            .with_custom_certificate_verifier(SkipServerVerification::new())
            .with_no_client_auth(),
    )?)));

    // connect to server
    let connection = endpoint
        .connect(server_addr, "localhost")
        .unwrap()
        .await
        .unwrap();
    println!("[client] connected: addr={}", connection.remote_address());

    Ok((endpoint, connection))
}

/// Dummy certificate verifier that treats any certificate as valid.
/// NOTE, such verification is vulnerable to MITM attacks, but convenient for testing.
#[derive(Debug)]
struct SkipServerVerification(Arc<rustls::crypto::CryptoProvider>);

impl SkipServerVerification {
    fn new() -> Arc<Self> {
        Arc::new(Self(Arc::new(rustls::crypto::ring::default_provider())))
    }
}

impl rustls::client::danger::ServerCertVerifier for SkipServerVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp: &[u8],
        _now: UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls12_signature(
            message,
            cert,
            dss,
            &self.0.signature_verification_algorithms,
        )
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls13_signature(
            message,
            cert,
            dss,
            &self.0.signature_verification_algorithms,
        )
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        self.0.signature_verification_algorithms.supported_schemes()
    }
}

pub fn serialize_client_message(
    message: &ReliableClientMessage,
) -> Result<Vec<u8>, Box<dyn Error + Send + Sync + 'static>> {
    let serialized_message = rkyv::to_bytes::<rancor::Error>(message);
    match serialized_message {
        Ok(bytes) => {
            // create the header with delimiter and size
            let size: MessageSize = (bytes.len() as u32).to_be_bytes();
            // attach the start delimiter to the header (this lets the server know that a new message is coming)
            let header = [&crate::DELIMITER[..], &size[..]].concat();
            // prepend the header to the serialized message
            let serialized_message = [&header, bytes.as_slice()].concat();
            return Ok(serialized_message);
        }
        Err(e) => return Err(Box::new(e)),
    }
}

async fn read_reliable_server_message(
    mut recv_stream: quinn::RecvStream,
    cancel_rev: watch::Receiver<bool>,
    server_message_sender: async_channel::Sender<ReliableServerMessage>,
) {
        println!("[client] start receiving messages from server");
        loop {
            // break loop if the cancel receiver is set to true
            if *cancel_rev.borrow() {
                println!("[client] cancel receiver is set to true, stopping receiving messages from server");
                break;
            }

            // get message from server

            // Read the delimiter first (to see that our frame has started)
            let mut delimiter_buf = [0u8; 1];
            if let Err(e) = recv_stream.read_exact(&mut delimiter_buf).await {
                println!("[client] failed to read delimiter: {e}");
                continue;
            }
            if delimiter_buf != crate::DELIMITER {
                println!("[client] received invalid delimiter: {:?}", delimiter_buf);
                continue;
            }
            // Read the size of the message
            let mut size_buf: MessageSize = [0u8; 4];
            if let Err(e) = recv_stream.read_exact(&mut size_buf).await {
                println!("[client] failed to read size: {e}");
                continue;
            }
            // Convert the size from bytes to u32
            let size = u32::from_be_bytes(size_buf);
            // then read the actual message (only read as much as we need)
            let mut buf = vec![0u8; size as usize];
            if let Ok(()) = recv_stream.read_exact(&mut buf).await {
                //println!("bytes read: {:?}", buf);
                let server_message =
                    rkyv::from_bytes::<ReliableServerMessage, rancor::Error>(&buf).unwrap();
                if let Err(send_error) = server_message_sender.send(server_message).await {
                    println!(
                        "[client] failed to send message to server receiver: {}",
                        send_error
                    );
                }
            } else {
                println!(
                    "[client] failed to receive message from server, stream id: {}",
                    recv_stream.id()
                );
            }
        }
}

async fn send_reliable_client_message(
    mut send_stream: quinn::SendStream,
    cancel_rev: watch::Receiver<bool>,
    client_message_receiver: async_channel::Receiver<ReliableClientMessage>,
) {
        println!(
            "[client] start sending messages to server, stream id: {}",
            send_stream.id()
        );
        loop {
            // break loop if the cancel receiver is set to true
            if *cancel_rev.borrow() {
                println!(
                    "[client] cancel receiver is set to true, stopping sending messages to server"
                );
                break;
            }
            // get message from sync client code
            while let Ok(message) = client_message_receiver.recv().await {
                let serialized_message = match serialize_client_message(&message) {
                    Ok(bytes) => bytes,
                    Err(e) => {
                        println!("[client] failed to serialize message: {}", e);
                        continue;
                    }
                };
                // then send the serialized message
                if let Ok(()) = send_stream.write_all(&serialized_message).await {
                } else {
                    println!("[client] failed to send message: {:?}", message);
                }
            }
        }
}

async fn connect_client_to_server(
    connection: Connection,
) -> Result<
    Client,
    Box<dyn Error + Send + Sync + 'static>,
> {
    let (cancel_sender, cancel_receiver) = watch::channel(false);
    println!("[client] connecting channel to server");
    let (mut send_stream, mut recv_stream) = connection
        .accept_bi()
        .await
        .map_err(|e| format!("Failed to accept bidirectional stream: {}", e))?;
    println!("[client] accepted bidirectional stream");
    // Create a channel for sending message to the server and receiving messages from it.
    let (reliable_server_sender, reliable_server_receiver) = async_channel::unbounded::<ReliableServerMessage>();
    // Create a channel for receiving messages from the tokio task to sync code
    let (reliable_client_sender, reliable_client_receiver) = async_channel::unbounded::<ReliableClientMessage>();
    // return handle to to the connection tasks so we can drop it later
    let mut join_set = tokio::task::JoinSet::new();
    let cancel_rev = cancel_receiver.clone();
    join_set.spawn(read_reliable_server_message(recv_stream, cancel_rev, reliable_server_sender));

    let cancel_rev = cancel_receiver.clone();
    join_set.spawn(send_reliable_client_message(send_stream, cancel_rev, reliable_client_receiver));

    Ok(Client {
        cancel_sender,
        server_receiver: reliable_server_receiver,
        client_sender: reliable_client_sender,
        join_set,
        local_player_id: PlayerId::default(),
    })
}

pub async fn run_client() -> Result<
    Client,
    Box<dyn Error + Send + Sync + 'static>,
> {
    let server_address = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8080);
    let (endpoint, connection) = connect_to_server(server_address).await?;
    let client = connect_client_to_server(connection).await?;
    Ok(client)
}

pub struct Client {
    pub cancel_sender: watch::Sender<bool>,
    pub server_receiver: async_channel::Receiver<ReliableServerMessage>,
    pub client_sender: async_channel::Sender<ReliableClientMessage>,
    pub join_set: tokio::task::JoinSet<()>,
    pub local_player_id: PlayerId,
}
