use std::{
    collections::HashMap,
    io::{Read, Write},
    path::PathBuf,
    thread,
};

use assertables::assert_ok;
use extensions_proto::moosync::types::{OpenClientFdRequest, ReadSockRequest, WriteSockRequest};
use interprocess::local_socket::{GenericFilePath, ListenerOptions, ToFsName, traits::Listener};
use tempdir::TempDir;
use tracing_test::traced_test;

use crate::socket::SocketManager;

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_socket_manager_disallowed_path() {
    let manager = SocketManager::new(None);

    let result = manager.open_client_fd(OpenClientFdRequest {
        path: "/tmp/disallowed.sock".to_string(),
    });

    assert_ok!(result.as_ref());
    assert_eq!(result.unwrap().sock_id, -1);
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_socket_manager_open_write_read_close() {
    let temp_dir = TempDir::new("sock_mgr_test").unwrap();
    let sock_file = temp_dir.path().join("ipc.sock");
    let listener_name = sock_file.clone().to_fs_name::<GenericFilePath>().unwrap();
    let listener = ListenerOptions::new()
        .name(listener_name)
        .create_sync()
        .unwrap();
    let server_handle = thread::spawn(move || {
        let mut conn = listener.accept().unwrap();
        let mut recv_buf = [0u8; 4];
        conn.read_exact(&mut recv_buf).unwrap();
        assert_eq!(&recv_buf, b"ping");
        conn.write_all(b"pong").unwrap();
    });
    let mut allowed = HashMap::new();
    allowed.insert(
        temp_dir.path().to_string_lossy().to_string(),
        PathBuf::from("/sandbox"),
    );
    let manager = SocketManager::new(Some(allowed));

    let open_res = manager
        .open_client_fd(OpenClientFdRequest {
            path: "/sandbox/ipc.sock".to_string(),
        })
        .unwrap();
    let write_res = manager
        .write_sock(WriteSockRequest {
            sock_id: open_res.sock_id,
            buf: b"ping".to_vec(),
        })
        .unwrap();
    let read_res = manager
        .read_sock(ReadSockRequest {
            sock_id: open_res.sock_id,
            read_len: 4,
        })
        .unwrap();
    manager.close_all();
    server_handle.join().unwrap();

    assert_eq!(open_res.sock_id, 0);
    assert_eq!(write_res.bytes_written, 4);
    assert_eq!(read_res.data, b"pong");
}
