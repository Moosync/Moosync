use std::{collections::HashMap, sync::Arc, time::Duration};

use extensions_proto::moosync::types::{
    ExtensionCommand, WsCloseRequest, WsConnectRequest, WsSendRequest, extension_command,
    ws_event_request, ws_send_request,
};
use futures_util::{SinkExt, StreamExt};
use tokio::{net::TcpListener, sync::mpsc, time::timeout};
use tokio_tungstenite::{
    accept_async,
    tungstenite::{
        Message,
        protocol::{CloseFrame, frame::coding::CloseCode},
    },
};

use crate::{
    errors::ExtensionError,
    websocket::{WsEventSink, WsManager},
};

struct ChannelSink {
    tx: mpsc::UnboundedSender<ExtensionCommand>,
}

#[async_trait::async_trait]
impl WsEventSink for ChannelSink {
    #[tracing::instrument(level = "debug", skip_all)]
    async fn dispatch(&self, command: ExtensionCommand) -> Result<(), ExtensionError> {
        let _ = self.tx.send(command);
        Ok(())
    }
}

fn create_test_manager() -> (WsManager, mpsc::UnboundedReceiver<ExtensionCommand>) {
    let (tx, rx) = mpsc::unbounded_channel();
    let manager = WsManager::with_sink(
        Some(vec!["127.0.0.1".to_string(), "localhost".to_string()]),
        "test.ext",
        Arc::new(ChannelSink { tx }),
    );
    (manager, rx)
}

#[tokio::test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_ws_host_not_permitted() {
    let (manager, _rx) = create_test_manager();

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
async fn test_ws_connect_send_and_pushed_receive() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let server_handle = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let mut server_ws = accept_async(stream).await.unwrap();
        let message = server_ws.next().await.unwrap().unwrap();
        assert_eq!(message.to_text().unwrap(), "ping");
        server_ws.send(Message::Text("pong".into())).await.unwrap();
        server_ws
            .send(Message::Binary(vec![1, 2, 3].into()))
            .await
            .unwrap();
        server_ws.flush().await.unwrap();
        let _ = server_ws.next().await;
    });
    let (manager, mut rx) = create_test_manager();

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
    let pushed_text = timeout(Duration::from_secs(2), rx.recv())
        .await
        .unwrap()
        .unwrap();
    let pushed_binary = timeout(Duration::from_secs(2), rx.recv())
        .await
        .unwrap()
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
    match pushed_text.event {
        Some(extension_command::Event::WsEvent(req)) => {
            assert_eq!(req.connection_id, connect_res.connection_id);
            match req.event {
                Some(ws_event_request::Event::Text(text)) => assert_eq!(text, "pong"),
                other => panic!("Unexpected ws_event: {:?}", other),
            }
        }
        other => panic!("Unexpected command: {:?}", other),
    }
    match pushed_binary.event {
        Some(extension_command::Event::WsEvent(req)) => {
            assert_eq!(req.connection_id, connect_res.connection_id);
            match req.event {
                Some(ws_event_request::Event::Binary(bin)) => assert_eq!(bin, vec![1, 2, 3]),
                other => panic!("Unexpected ws_event: {:?}", other),
            }
        }
        other => panic!("Unexpected command: {:?}", other),
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
        let _ = server_ws.next().await;
    });
    let (manager, mut rx) = create_test_manager();

    let connect_res = manager
        .ws_connect(WsConnectRequest {
            url: format!("ws://127.0.0.1:{}/immediate", port),
            protocols: vec![],
            headers: HashMap::new(),
        })
        .await
        .unwrap();
    let pushed = timeout(Duration::from_secs(2), rx.recv())
        .await
        .unwrap()
        .unwrap();
    let _ = manager
        .ws_close(WsCloseRequest {
            connection_id: connect_res.connection_id,
            code: Some(1000),
            reason: None,
        })
        .await
        .unwrap();
    server_handle.await.unwrap();

    match pushed.event {
        Some(extension_command::Event::WsEvent(req)) => {
            assert_eq!(req.connection_id, connect_res.connection_id);
            match req.event {
                Some(ws_event_request::Event::Text(text)) => assert_eq!(text, "hello"),
                other => panic!("Unexpected ws_event: {:?}", other),
            }
        }
        other => panic!("Unexpected command: {:?}", other),
    }
}

#[tokio::test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_ws_server_close_pushes_closed_event() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let server_handle = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let mut server_ws = accept_async(stream).await.unwrap();
        server_ws
            .send(Message::Close(Some(CloseFrame {
                code: CloseCode::Normal,
                reason: "bye".into(),
            })))
            .await
            .unwrap();
        server_ws.flush().await.unwrap();
    });
    let (manager, mut rx) = create_test_manager();

    let connect_res = manager
        .ws_connect(WsConnectRequest {
            url: format!("ws://127.0.0.1:{}/close", port),
            protocols: vec![],
            headers: HashMap::new(),
        })
        .await
        .unwrap();
    let pushed = timeout(Duration::from_secs(2), rx.recv())
        .await
        .unwrap()
        .unwrap();
    server_handle.await.unwrap();

    match pushed.event {
        Some(extension_command::Event::WsEvent(req)) => {
            assert_eq!(req.connection_id, connect_res.connection_id);
            match req.event {
                Some(ws_event_request::Event::Closed(closed)) => {
                    assert_eq!(closed.code, Some(1000));
                    assert_eq!(closed.reason.as_deref(), Some("bye"));
                    assert!(closed.error.is_none());
                }
                other => panic!("Unexpected ws_event: {:?}", other),
            }
        }
        other => panic!("Unexpected command: {:?}", other),
    }
}

#[tokio::test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_ws_ping_pong_handled_transparently() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let server_handle = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let mut server_ws = accept_async(stream).await.unwrap();
        server_ws
            .send(Message::Ping(vec![9, 8, 7].into()))
            .await
            .unwrap();
        server_ws.flush().await.unwrap();
        let pong = server_ws.next().await.unwrap().unwrap();
        assert_eq!(pong, Message::Pong(vec![9, 8, 7].into()));
        server_ws
            .send(Message::Text("after-pong".into()))
            .await
            .unwrap();
        server_ws.flush().await.unwrap();
        let _ = server_ws.next().await;
    });
    let (manager, mut rx) = create_test_manager();

    let connect_res = manager
        .ws_connect(WsConnectRequest {
            url: format!("ws://127.0.0.1:{}/ping", port),
            protocols: vec![],
            headers: HashMap::new(),
        })
        .await
        .unwrap();
    let pushed = timeout(Duration::from_secs(2), rx.recv())
        .await
        .unwrap()
        .unwrap();
    let _ = manager
        .ws_close(WsCloseRequest {
            connection_id: connect_res.connection_id,
            code: Some(1000),
            reason: None,
        })
        .await
        .unwrap();
    server_handle.await.unwrap();

    match pushed.event {
        Some(extension_command::Event::WsEvent(req)) => match req.event {
            Some(ws_event_request::Event::Text(text)) => assert_eq!(text, "after-pong"),
            other => panic!("Unexpected ws_event: {:?}", other),
        },
        other => panic!("Unexpected command: {:?}", other),
    }
}
