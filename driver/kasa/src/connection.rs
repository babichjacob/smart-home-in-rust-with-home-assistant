use std::{convert::Infallible, io, net::SocketAddr, num::NonZero, time::Duration};

use backoff::{ExponentialBackoff, ExponentialBackoffBuilder};
use protocol::light::{Kelvin, KelvinLight, Light, Rgb, RgbLight};
use snafu::{ResultExt, Snafu};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter},
    net::{TcpListener, TcpSocket, TcpStream},
    sync::{mpsc, oneshot, OnceCell},
    time::timeout,
};

use crate::messages::{GetSysInfo, GetSysInfoResponse, LB130USSys, SysInfo};

struct XorEncryption<const INITIAL_KEY: u8>;

impl<const INITIAL_KEY: u8> XorEncryption<INITIAL_KEY> {
    fn encrypt_in_place(bytes: &mut [u8]) {
        let mut key = INITIAL_KEY;
        for unencrypted_byte in bytes {
            let encrypted_byte = key ^ *unencrypted_byte;
            key = encrypted_byte;
            *unencrypted_byte = encrypted_byte;
        }
    }

    fn decrypt_in_place(bytes: &mut [u8]) {
        let mut key = INITIAL_KEY;
        for encrypted_byte in bytes {
            let unencrypted_byte = key ^ *encrypted_byte;
            key = *encrypted_byte;
            *encrypted_byte = unencrypted_byte;
        }
    }
}

fn into_encrypted(mut msg: Vec<u8>) -> Vec<u8> {
    let length = msg.len() as u32;
    let big_endian = length.to_be_bytes();
    XorEncryption::<171>::encrypt_in_place(&mut msg);

    let all_together = big_endian.into_iter().chain(msg);

    all_together.collect()
}

#[derive(Debug, Snafu)]
pub enum CommunicationError {
    SerializeError { source: serde_json::Error },
    WriteError { source: std::io::Error },
    ReadError { source: std::io::Error },
    DeserializeError { source: serde_json::Error },
    WrongDevice,
}

#[derive(Debug)]
enum LB130USMessage {
    GetSysInfo(oneshot::Sender<Result<LB130USSys, CommunicationError>>),
}

async fn lb130us_actor(
    addr: SocketAddr,
    disconnect_after_idle: Duration,
    mut messages: mpsc::Receiver<LB130USMessage>,
) {
    let mut connection_cell = None;

    loop {
        let (connection, message) = match &mut connection_cell {
            Some(connection) => match timeout(disconnect_after_idle, messages.recv()).await {
                Ok(Some(message)) => (connection, message),
                Ok(None) => return,
                Err(timed_out) => {
                    tracing::warn!(
                        ?addr,
                        ?timed_out,
                        "disconnecting from the LB130(US) because the idle timeout has been reached",
                    );

                    connection_cell.take();
                    continue;
                }
            },
            None => {
                let Some(message) = messages.recv().await else {
                    return;
                };

                tracing::info!(
                    "connecting for a first time / reconnecting after having gone idle..."
                );

                match backoff::future::retry_notify(
                    ExponentialBackoff::default(),
                    || async {
                        let stream = TcpStream::connect(addr).await?;
                        let (reader, writer) = stream.into_split();

                        let buf_reader = BufReader::new(reader);
                        let buf_writer = BufWriter::new(writer);

                        Ok((buf_reader, buf_writer))
                    },
                    |err, duration| {
                        tracing::error!(?err, ?duration);
                    },
                )
                .await
                {
                    Ok(connection) => (connection_cell.insert(connection), message),
                    Err(err) => {
                        tracing::error!(?addr, ?err, "error connecting to an LB130(US)");
                        continue;
                    }
                }
            }
        };

        let (reader, writer) = connection;

        tracing::info!("yay connected and got a message");

        // TODO: do something
        match message {
            LB130USMessage::GetSysInfo(callback) => {
                tracing::info!("going to try to get sys info for you...");

                // TODO: extract to its own function
                let outgoing = GetSysInfo;
                let outgoing = match serde_json::to_vec(&outgoing) {
                    Ok(outgoing) => outgoing,
                    Err(err) => {
                        // TODO (continued) instead of doing stuff like this
                        let _ =
                            callback.send(Err(CommunicationError::SerializeError { source: err }));
                        continue;
                    }
                };

                tracing::info!(?outgoing);

                let encrypted_outgoing = into_encrypted(outgoing);

                tracing::info!(?encrypted_outgoing);

                if let Err(err) = writer.write_all(&encrypted_outgoing).await {
                    connection_cell.take();
                    let _ = callback.send(Err(CommunicationError::WriteError { source: err }));
                    continue;
                }

                if let Err(err) = writer.flush().await {
                    connection_cell.take();
                    let _ = callback.send(Err(CommunicationError::WriteError { source: err }));
                    continue;
                }
                tracing::info!("sent it, now about to try to get a response");

                let incoming_length = match reader.read_u32().await {
                    Ok(incoming_length) => incoming_length,
                    Err(err) => {
                        connection_cell.take();
                        let _ = callback.send(Err(CommunicationError::ReadError { source: err }));
                        continue;
                    }
                };
                tracing::info!(?incoming_length);

                let mut incoming_message = Vec::new();
                incoming_message.resize(incoming_length as usize, 0);
                if let Err(err) = reader.read_exact(&mut incoming_message).await {
                    connection_cell.take();
                    let _ = callback.send(Err(CommunicationError::ReadError { source: err }));
                    continue;
                }

                XorEncryption::<171>::decrypt_in_place(&mut incoming_message);
                tracing::info!(?incoming_message);

                let response: GetSysInfoResponse = match serde_json::from_slice(&incoming_message) {
                    Ok(response) => response,
                    Err(err) => {
                        let _ = callback
                            .send(Err(CommunicationError::DeserializeError { source: err }));
                        continue;
                    }
                };
                tracing::info!(?response);

                let SysInfo::LB130US(lb130us) = response.system.get_sysinfo else {
                    let _ = callback.send(Err(CommunicationError::WrongDevice));
                    continue;
                };
                tracing::info!(?lb130us);

                let _ = callback.send(Ok(lb130us));
                tracing::info!("cool, gave a response! onto the next message!");
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct LB130USHandle {
    sender: mpsc::Sender<LB130USMessage>,
}

#[derive(Debug, Snafu)]
pub enum HandleError {
    CommunicationError { source: CommunicationError },
    Dead,
}

impl LB130USHandle {
    pub fn new(addr: SocketAddr, disconnect_after_idle: Duration, buffer: NonZero<usize>) -> Self {
        let (sender, receiver) = mpsc::channel(buffer.get());
        tokio::spawn(lb130us_actor(addr, disconnect_after_idle, receiver));
        Self { sender }
    }

    pub async fn get_sysinfo(&self) -> Result<LB130USSys, HandleError> {
        let (sender, receiver) = oneshot::channel();
        self.sender
            .send(LB130USMessage::GetSysInfo(sender))
            .await
            .map_err(|_| HandleError::Dead)?;
        receiver
            .await
            .map_err(|_| HandleError::Dead)?
            .context(CommunicationSnafu)
    }
}

impl Light for LB130USHandle {
    type IsOnError = Infallible; // TODO

    async fn is_on(&self) -> Result<bool, Self::IsOnError> {
        todo!()
    }

    type IsOffError = Infallible; // TODO

    async fn is_off(&self) -> Result<bool, Self::IsOffError> {
        todo!()
    }

    type TurnOnError = Infallible; // TODO

    async fn turn_on(&mut self) -> Result<(), Self::TurnOnError> {
        todo!()
    }

    type TurnOffError = Infallible; // TODO

    async fn turn_off(&mut self) -> Result<(), Self::TurnOffError> {
        todo!()
    }

    type ToggleError = Infallible; // TODO

    async fn toggle(&mut self) -> Result<(), Self::ToggleError> {
        todo!()
    }
}

impl KelvinLight for LB130USHandle {
    type TurnToKelvinError = Infallible; // TODO

    async fn turn_to_kelvin(&mut self, temperature: Kelvin) -> Result<(), Self::TurnToKelvinError> {
        todo!()
    }
}

impl RgbLight for LB130USHandle {
    type TurnToRgbError = Infallible; // TODO

    async fn turn_to_rgb(&mut self, color: Rgb) -> Result<(), Self::TurnToRgbError> {
        todo!()
    }
}
