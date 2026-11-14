use std::{
    collections::{HashMap, VecDeque},
    net::TcpStream,
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
use tungstenite::{
    Error as TsError, Message, WebSocket, client::IntoClientRequest, handshake::client::Request,
    protocol::CloseFrame,
};
use url::Url;

use crate::{context::extism_context::is_host_allowed, errors::ExtensionError};

struct ConnectionState {
    socket: WebSocket<TcpStream>,
    incoming: VecDeque<Message>,
    is_closed: bool,
}

pub struct ExtensionWsManager {
    connections: Arc<Mutex<HashMap<usize, Arc<Mutex<ConnectionState>>>>>,
    next_id: AtomicUsize,
    allowed_hosts: Option<Vec<String>>,
}

impl ExtensionWsManager {
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
        let parsed = Url::parse(raw_url)
            .map_err(|e| ExtensionError::WsFailedToConnect(raw_url.to_string(), e.to_string()))?;

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
    pub fn ws_connect(&self, req: WsConnectRequest) -> Result<WsConnectResponse, ExtensionError> {
        let parsed_url = self.validate_target_url(&req.url)?;
        let host = parsed_url.host_str().unwrap();
        let port = parsed_url.port().unwrap_or(80);

        let stream = TcpStream::connect((host, port)).map_err(|e| {
            ExtensionError::WsFailedToConnect(
                req.url.clone(),
                format!("Failed to connect to {}:{}: {}", host, port, e),
            )
        })?;
        stream
            .set_read_timeout(Some(Duration::from_millis(50)))
            .map_err(|e| ExtensionError::Ws(e.to_string()))?;

        let mut client_req: Request = req
            .url
            .as_str()
            .into_client_request()
            .map_err(|e| ExtensionError::WsFailedToConnect(req.url.clone(), e.to_string()))?;

        for (k, v) in &req.headers {
            if let (Ok(header_name), Ok(header_val)) = (
                tungstenite::http::header::HeaderName::from_bytes(k.as_bytes()),
                tungstenite::http::header::HeaderValue::from_str(v),
            ) {
                client_req.headers_mut().insert(header_name, header_val);
            }
        }

        if !req.protocols.is_empty() {
            let protocols_str = req.protocols.join(", ");
            if let Ok(val) = tungstenite::http::header::HeaderValue::from_str(&protocols_str) {
                client_req
                    .headers_mut()
                    .insert(tungstenite::http::header::SEC_WEBSOCKET_PROTOCOL, val);
            }
        }

        let (socket, _response) = tungstenite::client(client_req, stream).map_err(|e| {
            ExtensionError::WsFailedToConnect(
                req.url.clone(),
                format!("WebSocket handshake error: {}", e),
            )
        })?;

        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let state = ConnectionState {
            socket,
            incoming: VecDeque::new(),
            is_closed: false,
        };

        let mut connections = self.connections.lock().unwrap();
        connections.insert(id, Arc::new(Mutex::new(state)));

        Ok(WsConnectResponse {
            connection_id: id as u64,
        })
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn ws_send(&self, req: WsSendRequest) -> Result<WsSendResponse, ExtensionError> {
        let connection_id = req.connection_id as usize;
        let connection = {
            let connections = self.connections.lock().unwrap();
            connections
                .get(&connection_id)
                .cloned()
                .ok_or(ExtensionError::WsConnectionNotFound(connection_id))?
        };

        let mut state = connection.lock().unwrap();
        if state.is_closed {
            return Err(ExtensionError::WsConnectionClosed(connection_id));
        }

        let msg = match req.payload {
            Some(ws_send_request::Payload::Text(text)) => Message::Text(text.into()),
            Some(ws_send_request::Payload::Binary(bin)) => Message::Binary(bin.into()),
            None => return Err(ExtensionError::Ws("Missing payload".to_string())),
        };

        state
            .socket
            .send(msg)
            .map_err(|e| ExtensionError::WsFailedToSend(connection_id, e.to_string()))?;

        Ok(WsSendResponse { success: true })
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn poll_socket(state: &mut ConnectionState) {
        loop {
            match state.socket.read() {
                Ok(msg) => match msg {
                    Message::Text(_) | Message::Binary(_) => {
                        state.incoming.push_back(msg);
                    }
                    Message::Close(_) => {
                        state.is_closed = true;
                        break;
                    }
                    Message::Ping(data) => {
                        let _ = state.socket.send(Message::Pong(data));
                    }
                    Message::Pong(_) | Message::Frame(_) => {}
                },
                Err(TsError::Io(ref e))
                    if e.kind() == std::io::ErrorKind::WouldBlock
                        || e.kind() == std::io::ErrorKind::TimedOut =>
                {
                    break;
                }
                Err(TsError::ConnectionClosed | TsError::AlreadyClosed) => {
                    state.is_closed = true;
                    break;
                }
                Err(_) => {
                    break;
                }
            }
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn ws_receive(&self, req: WsReceiveRequest) -> Result<WsReceiveResponse, ExtensionError> {
        let connection_id = req.connection_id as usize;
        let connection = {
            let connections = self.connections.lock().unwrap();
            connections
                .get(&connection_id)
                .cloned()
                .ok_or(ExtensionError::WsConnectionNotFound(connection_id))?
        };

        let timeout = Duration::from_millis(req.timeout_ms.unwrap_or(50).max(10));
        let start = std::time::Instant::now();

        loop {
            let mut state = connection.lock().unwrap();
            if let Some(msg) = state.incoming.pop_front() {
                let payload = match msg {
                    Message::Text(t) => Some(ws_receive_response::Payload::Text(t.to_string())),
                    Message::Binary(b) => Some(ws_receive_response::Payload::Binary(b.to_vec())),
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

            Self::poll_socket(&mut state);
            if let Some(msg) = state.incoming.pop_front() {
                let payload = match msg {
                    Message::Text(t) => Some(ws_receive_response::Payload::Text(t.to_string())),
                    Message::Binary(b) => Some(ws_receive_response::Payload::Binary(b.to_vec())),
                    _ => None,
                };
                return Ok(WsReceiveResponse {
                    payload,
                    is_closed: state.is_closed,
                });
            }

            if start.elapsed() >= timeout {
                return Ok(WsReceiveResponse {
                    payload: None,
                    is_closed: state.is_closed,
                });
            }

            drop(state);
            std::thread::sleep(Duration::from_millis(5));
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn ws_close(&self, req: WsCloseRequest) -> Result<WsCloseResponse, ExtensionError> {
        let connection_id = req.connection_id as usize;
        let connection = {
            let mut connections = self.connections.lock().unwrap();
            connections.remove(&connection_id)
        };

        if let Some(conn) = connection {
            let mut state = conn.lock().unwrap();
            if !state.is_closed {
                state.is_closed = true;
                let close_frame = CloseFrame {
                    code: tungstenite::protocol::frame::coding::CloseCode::from(
                        req.code.unwrap_or(1000) as u16,
                    ),
                    reason: req.reason.map(|r| r.into()).unwrap_or_default(),
                };
                let _ = state.socket.close(Some(close_frame));
                let _ = state.socket.flush();
            }
        }

        Ok(WsCloseResponse { success: true })
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn close_all(&self) {
        let mut connections = self.connections.lock().unwrap();
        for (_, conn) in connections.drain() {
            let mut state = conn.lock().unwrap();
            if !state.is_closed {
                state.is_closed = true;
                let _ = state.socket.close(None);
                let _ = state.socket.flush();
            }
        }
    }
}
