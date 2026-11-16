use std::{
    collections::{HashMap, VecDeque},
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

use extensions_proto::moosync::types::{
    WsCloseRequest, WsCloseResponse, WsConnectRequest, WsConnectResponse, WsReceiveRequest,
    WsReceiveResponse, WsSendRequest, WsSendResponse, ws_receive_response, ws_send_request,
};
use futures_util::{SinkExt, StreamExt};
use tokio::{net::TcpStream, runtime::Handle, sync::Mutex as AsyncMutex, time::timeout};
use tokio_tungstenite::{
    WebSocketStream, client_async,
    tungstenite::{
        Message,
        client::IntoClientRequest,
        handshake::client::Request,
        http::header::{HeaderName, HeaderValue, SEC_WEBSOCKET_PROTOCOL},
        protocol::{CloseFrame, frame::coding::CloseCode},
    },
};
use tracing::Instrument;
use url::Url;

use crate::{errors::ExtensionError, http::is_host_allowed};

struct ConnectionState {
    socket: WebSocketStream<TcpStream>,
    incoming: VecDeque<Message>,
    is_closed: bool,
}

pub struct WsManager {
    connections: Arc<Mutex<HashMap<usize, Arc<AsyncMutex<ConnectionState>>>>>,
    next_id: AtomicUsize,
    allowed_hosts: Option<Vec<String>>,
}

impl WsManager {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(allowed_hosts: Option<Vec<String>>) -> Self {
        Self {
            connections: Arc::new(Mutex::new(HashMap::new())),
            next_id: AtomicUsize::new(1),
            allowed_hosts,
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn validate_target_url(&self, raw_url: &str) -> Result<Url, ExtensionError> {
        let parsed = Url::parse(raw_url)?;

        if parsed.scheme() != "ws" {
            return Err(ExtensionError::WsFailedToConnect(
                raw_url.to_string(),
                "Only ws:// scheme is currently supported for local extensions".to_string(),
            ));
        }

        let host = parsed.host_str().ok_or_else(|| {
            ExtensionError::WsFailedToConnect(raw_url.to_string(), "Missing host".to_string())
        })?;

        if !is_host_allowed(host, self.allowed_hosts.as_deref()) {
            return Err(ExtensionError::WsHostNotPermitted(host.to_string()));
        }

        Ok(parsed)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn ws_connect(
        &self,
        request: WsConnectRequest,
    ) -> Result<WsConnectResponse, ExtensionError> {
        let parsed_url = self.validate_target_url(&request.url)?;
        let host = parsed_url.host_str().unwrap().to_string();
        let port = parsed_url.port().unwrap_or(80);
        let stream = TcpStream::connect((host.as_str(), port)).await?;

        let mut client_req: Request = request.url.as_str().into_client_request()?;
        for (header_key, header_val) in &request.headers {
            if let (Ok(header_name), Ok(parsed_val)) = (
                HeaderName::from_bytes(header_key.as_bytes()),
                HeaderValue::from_str(header_val),
            ) {
                client_req.headers_mut().insert(header_name, parsed_val);
            }
        }

        if !request.protocols.is_empty() {
            let protocols_str = request.protocols.join(", ");
            if let Ok(val) = HeaderValue::from_str(&protocols_str) {
                client_req.headers_mut().insert(SEC_WEBSOCKET_PROTOCOL, val);
            }
        }

        let (socket, _response) = client_async(client_req, stream).await?;
        let connection_id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let state = ConnectionState {
            socket,
            incoming: VecDeque::new(),
            is_closed: false,
        };

        let mut connections = self.connections.lock().unwrap();
        connections.insert(connection_id, Arc::new(AsyncMutex::new(state)));

        Ok(WsConnectResponse {
            connection_id: connection_id as u64,
        })
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn ws_send(&self, request: WsSendRequest) -> Result<WsSendResponse, ExtensionError> {
        let connection_id = request.connection_id as usize;
        let connection = {
            let connections = self.connections.lock().unwrap();
            connections
                .get(&connection_id)
                .cloned()
                .ok_or(ExtensionError::WsConnectionNotFound(connection_id))?
        };

        let message = match request.payload {
            Some(ws_send_request::Payload::Text(text)) => Message::Text(text.into()),
            Some(ws_send_request::Payload::Binary(bin)) => Message::Binary(bin.into()),
            None => return Err(ExtensionError::Ws("Missing payload".to_string())),
        };

        let mut state = connection.lock().await;
        if state.is_closed {
            return Err(ExtensionError::WsConnectionClosed(connection_id));
        }
        state.socket.send(message).await?;
        Ok(WsSendResponse { success: true })
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn ws_receive(
        &self,
        request: WsReceiveRequest,
    ) -> Result<WsReceiveResponse, ExtensionError> {
        let connection_id = request.connection_id as usize;
        let connection = {
            let connections = self.connections.lock().unwrap();
            connections
                .get(&connection_id)
                .cloned()
                .ok_or(ExtensionError::WsConnectionNotFound(connection_id))?
        };

        let wait_duration = Duration::from_millis(request.timeout_ms.unwrap_or(50).max(10));
        let mut state = connection.lock().await;
        if let Some(message) = state.incoming.pop_front() {
            let payload = match message {
                Message::Text(text) => Some(ws_receive_response::Payload::Text(text.to_string())),
                Message::Binary(bin) => Some(ws_receive_response::Payload::Binary(bin.to_vec())),
                _ => None,
            };
            return Ok(WsReceiveResponse {
                payload,
                is_closed: state.is_closed,
            });
        }

        if state.is_closed {
            return Ok(WsReceiveResponse {
                payload: None,
                is_closed: true,
            });
        }

        let deadline = tokio::time::Instant::now() + wait_duration;
        loop {
            let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
            if remaining.is_zero() {
                return Ok(WsReceiveResponse {
                    payload: None,
                    is_closed: state.is_closed,
                });
            }

            match timeout(remaining, state.socket.next()).await {
                Ok(Some(Ok(message))) => match message {
                    Message::Text(text) => {
                        return Ok(WsReceiveResponse {
                            payload: Some(ws_receive_response::Payload::Text(text.to_string())),
                            is_closed: state.is_closed,
                        });
                    }
                    Message::Binary(bin) => {
                        return Ok(WsReceiveResponse {
                            payload: Some(ws_receive_response::Payload::Binary(bin.to_vec())),
                            is_closed: state.is_closed,
                        });
                    }
                    Message::Close(_) => {
                        state.is_closed = true;
                        return Ok(WsReceiveResponse {
                            payload: None,
                            is_closed: true,
                        });
                    }
                    Message::Ping(data) => {
                        let _ = state.socket.send(Message::Pong(data)).await;
                    }
                    Message::Pong(_) | Message::Frame(_) => {}
                },
                Ok(Some(Err(_))) | Ok(None) => {
                    state.is_closed = true;
                    return Ok(WsReceiveResponse {
                        payload: None,
                        is_closed: true,
                    });
                }
                Err(_) => {
                    return Ok(WsReceiveResponse {
                        payload: None,
                        is_closed: state.is_closed,
                    });
                }
            }
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn ws_close(
        &self,
        request: WsCloseRequest,
    ) -> Result<WsCloseResponse, ExtensionError> {
        let connection_id = request.connection_id as usize;
        let connection = {
            let mut connections = self.connections.lock().unwrap();
            connections.remove(&connection_id)
        };

        if let Some(conn) = connection {
            let code = request.code.unwrap_or(1000) as u16;
            let mut state = conn.lock().await;
            if !state.is_closed {
                state.is_closed = true;
                let close_frame = CloseFrame {
                    code: CloseCode::from(code),
                    reason: request.reason.map(|r| r.into()).unwrap_or_default(),
                };
                let _ = state.socket.close(Some(close_frame)).await;
                let _ = state.socket.flush().await;
            }
        }

        Ok(WsCloseResponse { success: true })
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn close_all(&self) {
        let drained: Vec<Arc<AsyncMutex<ConnectionState>>> = {
            let mut connections = self.connections.lock().unwrap();
            connections.drain().map(|(_, conn)| conn).collect()
        };

        if drained.is_empty() {
            return;
        }

        if let Ok(handle) = Handle::try_current() {
            handle.spawn(
                async move {
                    for conn in drained {
                        let mut state = conn.lock().await;
                        if !state.is_closed {
                            state.is_closed = true;
                            let _ = state.socket.close(None).await;
                            let _ = state.socket.flush().await;
                        }
                    }
                }
                .in_current_span(),
            );
        }
    }
}
