use std::collections::HashMap;

use extensions_proto::moosync::types::{
    WsCloseRequest, WsConnectRequest, WsReceiveRequest, WsSendRequest, ws_receive_response,
    ws_send_request,
};
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio_tungstenite::{accept_async, tungstenite::Message};

use crate::{errors::ExtensionError, websocket::WsManager};

#[tokio::test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_ws_host_not_permitted() {
    let manager = WsManager::new(Some(vec!["localhost".to_string()]));

    let result = manager
        .ws_connect(WsConnectRequest {
            url: "ws://example.com:9090".to_string(),
            protocols: vec![],
            headers: HashMap::new(),
        })
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        ExtensionError::WsHostNotPermitted(host) => {
            assert_eq!(host, "example.com");
        }
        other => panic!("Unexpected error: {:?}", other),
    }
}

#[tokio::test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_ws_connect_send_receive_close() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let server_handle = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let mut server_ws = accept_async(stream).await.unwrap();
        let message = server_ws.next().await.unwrap().unwrap();
        assert_eq!(message.to_text().unwrap(), "ping");
        server_ws.send(Message::Text("pong".into())).await.unwrap();
        server_ws.flush().await.unwrap();
        let _ = server_ws.next().await;
    });
    let manager = WsManager::new(Some(vec!["127.0.0.1".to_string()]));

    let connect_res = manager
        .ws_connect(WsConnectRequest {
            url: format!("ws://127.0.0.1:{}/test", port),
            protocols: vec![],
            headers: HashMap::new(),
        })
        .await
        .unwrap();
    let send_res = manager
        .ws_send(WsSendRequest {
            connection_id: connect_res.connection_id,
            payload: Some(ws_send_request::Payload::Text("ping".to_string())),
        })
        .await
        .unwrap();
    let recv_res = manager
        .ws_receive(WsReceiveRequest {
            connection_id: connect_res.connection_id,
            timeout_ms: Some(500),
        })
        .await
        .unwrap();
    let close_res = manager
        .ws_close(WsCloseRequest {
            connection_id: connect_res.connection_id,
            code: Some(1000),
            reason: Some("normal".to_string()),
        })
        .await
        .unwrap();
    server_handle.await.unwrap();

    assert!(send_res.success);
    assert!(close_res.success);
    match recv_res.payload {
        Some(ws_receive_response::Payload::Text(text)) => assert_eq!(text, "pong"),
        other => panic!("Unexpected payload: {:?}", other),
    }
}

#[tokio::test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_ws_handshake_with_immediate_frame() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let server_handle = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let mut server_ws = accept_async(stream).await.unwrap();
        server_ws.send(Message::Text("hello".into())).await.unwrap();
        server_ws.flush().await.unwrap();
    });
    let manager = WsManager::new(Some(vec!["127.0.0.1".to_string()]));

    let connect_res = manager
        .ws_connect(WsConnectRequest {
            url: format!("ws://127.0.0.1:{}/immediate", port),
            protocols: vec![],
            headers: HashMap::new(),
        })
        .await
        .unwrap();
    let recv_res = manager
        .ws_receive(WsReceiveRequest {
            connection_id: connect_res.connection_id,
            timeout_ms: Some(500),
        })
        .await
        .unwrap();
    server_handle.await.unwrap();

    match recv_res.payload {
        Some(ws_receive_response::Payload::Text(text)) => assert_eq!(text, "hello"),
        other => panic!("Unexpected payload: {:?}", other),
    }
}
