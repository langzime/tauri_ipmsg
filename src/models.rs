use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct OnlineUser {
    pub name: String,
    pub group: String,
    pub host: String,
    pub addr: SocketAddr,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct FileInfo {
    pub packet_no: u32,
    pub file_id: u32,
    pub name: String,
    pub size: u64,
    pub saved: bool,
    pub received: u64,
    pub is_dir: bool,
    pub local_path: Option<String>,
    pub current_file: Option<String>,
    #[serde(default)]
    pub error: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ChatMessage {
    pub from: SocketAddr,
    pub to: SocketAddr,
    pub is_me: bool,
    pub text: String,
    pub time: String,
    pub file: Option<FileInfo>,
}
