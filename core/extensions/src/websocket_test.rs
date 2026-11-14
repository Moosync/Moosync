use std::{collections::HashMap, net::TcpListener, thread};

use extensions_proto::moosync::types::{
    WsCloseRequest, WsConnectRequest, WsReceiveRequest, WsSendRequest, ws_receive_response,
    ws_send_request,
};

use crate::websocket::ExtensionWsManager;

#[test]
fn test_ws_host_not_permitted() {
    let mgr = ExtensionWsManager::new(Some(vec!["localhost".to_string()]));

    let res = mgr.ws_connect(WsConnectRequest {
        url: "ws://example.com:9090".to_string(),
        protocols: vec![],
        headers: HashMap::new(),
    });

    assert!(res.is_err());
    match res.unwrap_err() {
        crate::ExtensionError::WsHostNotPermitted(host) => {
            assert_eq!(host, "example.com");
        }
        other => panic!("Unexpected error: {:?}", other),
    }
}

#[test]
fn test_ws_connect_send_receive_close() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();

    let server_handle = thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        let mut server_ws = tungstenite::accept(stream).unwrap();

        let msg = server_ws.read().unwrap();
        assert_eq!(msg.to_text().unwrap(), "ping");

        server_ws
            .send(tungstenite::Message::Text("pong".into()))
            .unwrap();
        server_ws.flush().unwrap();

        let _ = server_ws.read();
    });

    let mgr = ExtensionWsManager::new(Some(vec!["127.0.0.1".to_string()]));

    let connect_res = mgr.ws_connect(WsConnectRequest {
        url: format!("ws://127.0.0.1:{}/test", port),
        protocols: vec![],
        headers: HashMap::new(),
    });

    assert!(connect_res.is_ok());
    let conn_id = connect_res.unwrap().connection_id;

    let send_res = mgr.ws_send(WsSendRequest {
        connection_id: conn_id,
        payload: Some(ws_send_request::Payload::Text("ping".to_string())),
    });

    assert!(send_res.is_ok());

    let recv_res = mgr.ws_receive(WsReceiveRequest {
        connection_id: conn_id,
        timeout_ms: Some(500),
    });

    assert!(recv_res.is_ok());
    let recv = recv_res.unwrap();
    match recv.payload {
        Some(ws_receive_response::Payload::Text(t)) => assert_eq!(t, "pong"),
        other => panic!("Unexpected payload: {:?}", other),
    }

    let close_res = mgr.ws_close(WsCloseRequest {
        connection_id: conn_id,
        code: Some(1000),
        reason: Some("normal".to_string()),
    });

    assert!(close_res.is_ok());

    server_handle.join().unwrap();
}

#[test]
fn test_ws_handshake_with_immediate_frame() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();

    let server_handle = thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        let mut server_ws = tungstenite::accept(stream).unwrap();
        server_ws
            .send(tungstenite::Message::Text("hello".into()))
            .unwrap();
        server_ws.flush().unwrap();
    });

    let mgr = ExtensionWsManager::new(Some(vec!["127.0.0.1".to_string()]));

    let connect_res = mgr.ws_connect(WsConnectRequest {
        url: format!("ws://127.0.0.1:{}/immediate", port),
        protocols: vec![],
        headers: HashMap::new(),
    });

    assert!(connect_res.is_ok());
    let conn_id = connect_res.unwrap().connection_id;

    let recv_res = mgr.ws_receive(WsReceiveRequest {
        connection_id: conn_id,
        timeout_ms: Some(500),
    });

    assert!(recv_res.is_ok());
    let recv = recv_res.unwrap();
    match recv.payload {
        Some(ws_receive_response::Payload::Text(t)) => assert_eq!(t, "hello"),
        other => panic!("Unexpected payload: {:?}", other),
    }

    server_handle.join().unwrap();
}
