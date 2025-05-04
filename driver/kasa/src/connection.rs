use crate::messages::{
    GetSysInfo, GetSysInfoResponse, LB130USSys, LightState, Off, On, SetLightLastOn, SetLightOff,
    SetLightState, SetLightStateArgs, SetLightStateResponse, SetLightTo, SysInfo,
};
use backon::{FibonacciBuilder, Retryable};

use serde::{Deserialize, Serialize};
use snafu::{ResultExt, Snafu};
use std::{io, net::SocketAddr, num::NonZero, time::Duration};
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader, BufWriter},
    net::TcpStream,
    sync::{mpsc, oneshot},
    time::timeout,
};

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

fn should_try_reconnecting(communication_error: &CommunicationError) -> bool {
    matches!(
        communication_error,
        CommunicationError::WriteError { .. } | CommunicationError::ReadError { .. }
    )
}

#[derive(Debug)]
enum LB130USMessage {
    GetSysInfo(oneshot::Sender<Result<LB130USSys, CommunicationError>>),
    SetLightState(
        SetLightStateArgs,
        oneshot::Sender<Result<SetLightStateResponse, CommunicationError>>,
    ),
}

#[tracing::instrument(skip(messages))]
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

                match (|| async {
                    let stream = TcpStream::connect(addr).await?;
                    let (reader, writer) = stream.into_split();

                    let buf_reader = BufReader::new(reader);
                    let buf_writer = BufWriter::new(writer);

                    Ok((buf_reader, buf_writer))
                })
                .retry(FibonacciBuilder::default())
                .notify(|err: &io::Error, duration| {
                    tracing::error!(?err, ?duration);
                })
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

        match message {
            LB130USMessage::GetSysInfo(callback) => {
                let res = handle_get_sysinfo(writer, reader).await;

                if let Err(communication_error) = &res {
                    if should_try_reconnecting(communication_error) {
                        connection_cell.take();
                    }
                }

                let _ = callback.send(res);
            }
            LB130USMessage::SetLightState(args, callback) => {
                let res = handle_set_light_state(writer, reader, args).await;

                if let Err(communication_error) = &res {
                    if should_try_reconnecting(communication_error) {
                        connection_cell.take();
                    }
                }

                let _ = callback.send(res);
            }
        }
    }
}

#[tracing::instrument(skip(writer, reader, request))]
async fn send_request<
    AW: AsyncWrite + Unpin,
    AR: AsyncRead + Unpin,
    Request: Serialize,
    Response: for<'de> Deserialize<'de>,
>(
    writer: &mut AW,
    reader: &mut AR,
    request: &Request,
) -> Result<Response, CommunicationError> {
    let outgoing = serde_json::to_vec(request).context(SerializeSnafu)?;
    tracing::info!(?outgoing);

    let encrypted_outgoing = into_encrypted(outgoing);
    tracing::info!(?encrypted_outgoing);

    writer
        .write_all(&encrypted_outgoing)
        .await
        .context(WriteSnafu)?;
    writer.flush().await.context(WriteSnafu)?;
    tracing::info!("sent it, now about to try to get a response");

    let incoming_length = reader.read_u32().await.context(ReadSnafu)?;
    tracing::info!(?incoming_length);

    let mut incoming_message = Vec::new();
    incoming_message.resize(incoming_length as usize, 0);
    reader
        .read_exact(&mut incoming_message)
        .await
        .context(ReadSnafu)?;

    XorEncryption::<171>::decrypt_in_place(&mut incoming_message);
    tracing::info!(?incoming_message);

    let response_as_json: serde_json::Value =
        serde_json::from_slice(&incoming_message).context(DeserializeSnafu)?;
    tracing::info!(?response_as_json);

    let response = Response::deserialize(response_as_json).context(DeserializeSnafu)?;

    Ok(response)
}

#[tracing::instrument(skip(writer, reader))]
async fn handle_get_sysinfo<AW: AsyncWrite + Unpin, AR: AsyncRead + Unpin>(
    writer: &mut AW,
    reader: &mut AR,
) -> Result<LB130USSys, CommunicationError> {
    let request = GetSysInfo;
    let response: GetSysInfoResponse = send_request(writer, reader, &request).await?;

    let SysInfo::LB130US(lb130us) = response.system.get_sysinfo else {
        return Err(CommunicationError::WrongDevice);
    };
    tracing::info!(?lb130us);

    Ok(lb130us)
}

#[tracing::instrument(skip(writer, reader))]
async fn handle_set_light_state<AW: AsyncWrite + Unpin, AR: AsyncRead + Unpin>(
    writer: &mut AW,
    reader: &mut AR,
    args: SetLightStateArgs,
) -> Result<SetLightStateResponse, CommunicationError> {
    let request = SetLightState(args);
    send_request(writer, reader, &request).await
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

    pub async fn set_light_state(
        &self,
        args: SetLightStateArgs,
    ) -> Result<SetLightStateResponse, HandleError> {
        let (sender, receiver) = oneshot::channel();
        self.sender
            .send(LB130USMessage::SetLightState(args, sender))
            .await
            .map_err(|_| HandleError::Dead)?;
        receiver
            .await
            .map_err(|_| HandleError::Dead)?
            .context(CommunicationSnafu)
    }
}
