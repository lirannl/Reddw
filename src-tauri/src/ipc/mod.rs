use reddw_ipc::IPCMessage;
use tokio::sync::watch::Receiver;

pub type IPCReceiver = Receiver<(IPCMessage, Vec<u8>)>;