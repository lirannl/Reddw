use anyhow::Result;
use lazy_static::lazy_static;
use rmp_serde::to_vec;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::io::AsyncWriteExt;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum IPCMessage {
    Init,
    AutomationSocket,
    PluginResponse(String),
}

pub type IPCData<T> = (IPCMessage, T);

static SOCKET_ID: &str = include_str!("../../automation_socket.txt");

#[cfg(target_family = "unix")]
lazy_static! {
    pub static ref SOCKET_PATH: PathBuf = format!(
        "/tmp/reddw-{SOCKET_ID}-{}.sock",
        hex::encode(whoami::username())
    )
    .into();
}

#[cfg(target_family = "unix")]
pub async fn message_ipc<T: Serialize>(message: IPCData<T>) -> Result<()> {
    let mut stream = tokio::net::UnixStream::connect(SOCKET_PATH.as_path()).await?;
    let writer = &mut stream;
    writer
        .write_all(&to_vec(&(message.0, to_vec(&message.1)?))?)
        .await?;
    Ok(())
}
