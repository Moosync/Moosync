use std::{
    collections::HashMap,
    io::{Read, Write},
    path::{Path, PathBuf},
    str::FromStr,
    sync::{Arc, Mutex},
};

use extensions_proto::moosync::types::{
    OpenClientFdRequest, OpenClientFdResponse, ReadSockRequest, ReadSockResponse, WriteSockRequest,
    WriteSockResponse,
};
use interprocess::local_socket::{
    GenericFilePath, GenericNamespaced, NameType, Stream as LocalSocketStream, ToFsName, ToNsName,
    traits::Stream,
};

use crate::errors::ExtensionError;

pub struct SocketManager {
    socks: Arc<Mutex<Vec<LocalSocketStream>>>,
    allowed_paths: Option<HashMap<String, PathBuf>>,
}

#[tracing::instrument(level = "debug", skip_all)]
fn try_connect_allowed_path(
    host_key: &str,
    guest_value: &Path,
    sock_path: &str,
) -> Option<LocalSocketStream> {
    let Some(allowed_path) = guest_value.to_str() else {
        tracing::error!("Failed to convert mapped path: {:?} to string", guest_value);
        return None;
    };

    if !sock_path.starts_with(allowed_path) {
        return None;
    }

    let relative = sock_path.replacen(allowed_path, "", 1);
    let Ok(mapped_path) = PathBuf::from_str(&format!("{host_key}/{relative}"));
    if !mapped_path.exists() {
        tracing::debug!("Path {:?} does not exist", mapped_path);
        return None;
    }

    let mapped_name = if GenericNamespaced::is_supported() && host_key.starts_with("\\\\.\\pipe\\")
    {
        mapped_path
            .file_name()
            .unwrap()
            .to_ns_name::<GenericNamespaced>()
    } else {
        mapped_path.to_fs_name::<GenericFilePath>()
    };

    match mapped_name {
        Ok(name) => match LocalSocketStream::connect(name) {
            Ok(sock) => Some(sock),
            Err(error) => {
                tracing::error!("Failed to connect to local socket: {}", error);
                None
            }
        },
        Err(error) => {
            tracing::error!("Failed to resolve local socket name: {}", error);
            None
        }
    }
}

impl SocketManager {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(allowed_paths: Option<HashMap<String, PathBuf>>) -> Self {
        Self {
            socks: Arc::new(Mutex::new(Vec::new())),
            allowed_paths,
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn open_client_fd(
        &self,
        request: OpenClientFdRequest,
    ) -> Result<OpenClientFdResponse, ExtensionError> {
        let mut socks = self.socks.lock().unwrap();
        if socks.len() > u8::MAX as usize {
            tracing::error!("Cannot open more sockets");
            return Ok(OpenClientFdResponse { sock_id: -1 });
        }

        let Some(allowed_paths) = self.allowed_paths.as_ref() else {
            tracing::error!("Not enough permissions to access {}", request.path);
            return Ok(OpenClientFdResponse { sock_id: -1 });
        };

        let Ok(sock_path_parsed) = PathBuf::from_str(request.path.as_str());
        let Some(sock_path_str) = sock_path_parsed.to_str() else {
            tracing::error!("Failed to convert passed path to string");
            return Ok(OpenClientFdResponse { sock_id: -1 });
        };

        let Some(sock) = allowed_paths.iter().find_map(|(host_key, guest_value)| {
            try_connect_allowed_path(host_key, guest_value, sock_path_str)
        }) else {
            tracing::error!("Sock path not specified in allowed_paths");
            return Ok(OpenClientFdResponse { sock_id: -1 });
        };

        socks.push(sock);
        Ok(OpenClientFdResponse {
            sock_id: (socks.len() - 1) as i64,
        })
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn write_sock(
        &self,
        request: WriteSockRequest,
    ) -> Result<WriteSockResponse, ExtensionError> {
        let mut socks = self.socks.lock().unwrap();

        let Some(sock) = socks.get_mut(request.sock_id as usize) else {
            tracing::error!("Invalid sock id {}", request.sock_id);
            return Ok(WriteSockResponse { bytes_written: -1 });
        };

        if let Err(error) = sock.write_all(&request.buf) {
            tracing::error!("Failed to write data to sock {}", error);
            return Ok(WriteSockResponse { bytes_written: -1 });
        }

        Ok(WriteSockResponse {
            bytes_written: request.buf.len() as i64,
        })
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn read_sock(&self, request: ReadSockRequest) -> Result<ReadSockResponse, ExtensionError> {
        let mut socks = self.socks.lock().unwrap();

        let Some(sock) = socks.get_mut(request.sock_id as usize) else {
            tracing::error!("Invalid sock id {}", request.sock_id);
            return Ok(ReadSockResponse { data: vec![] });
        };

        let mut read_len = request.read_len;
        if read_len == 0 || read_len > 1024 {
            read_len = 1024;
        }

        let mut buffer = vec![0; read_len as usize];
        let read_count = match sock.read(&mut buffer) {
            Ok(read_count) => read_count,
            Err(error) => {
                tracing::error!("Failed to read from socket: {}", error);
                return Ok(ReadSockResponse { data: vec![] });
            }
        };

        if read_count > 1024 {
            tracing::error!("Read out of bounds");
            return Ok(ReadSockResponse { data: vec![] });
        }
        buffer.truncate(read_count);
        Ok(ReadSockResponse { data: buffer })
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn close_all(&self) {
        let mut socks = self.socks.lock().unwrap();
        socks.clear();
    }
}
