use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex, Weak,
        atomic::{AtomicUsize, Ordering},
    },
};

use extensions_proto::moosync::types::{
    ExtensionCommand, WsCloseRequest, WsCloseResponse, WsClosed, WsConnectRequest,
    WsConnectResponse, WsEventRequest, WsSendRequest, WsSendResponse, extension_command,
    ws_event_request, ws_send_request,
};
use futures_util::{
    SinkExt, StreamExt,
    stream::{SplitSink, SplitStream},
};
use tokio::{
    net::TcpStream,
    runtime::Handle,
    sync::{Mutex as AsyncMutex, mpsc, oneshot},
};
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

use crate::{errors::ExtensionError, extension::AsyncContext, http::is_host_allowed};

#[async_trait::async_trait]
pub trait WsEventSink: Send + Sync {
    async fn dispatch(&self, command: ExtensionCommand) -> Result<(), ExtensionError>;
}

struct ContextWsEventSink {
    context: Weak<AsyncContext>,
}

#[async_trait::async_trait]
impl WsEventSink for ContextWsEventSink {
    #[tracing::instrument(level = "debug", skip_all)]
    async fn dispatch(&self, command: ExtensionCommand) -> Result<(), ExtensionError> {
        let Some(context) = self.context.upgrade() else {
            return Err(ExtensionError::NoContext);
        };
        let extension_context = context.wait().await?;
        let _ = extension_context.execute_command(command).await?;
        Ok(())
    }
}

enum WsIncomingEvent {
    Text(String),
    Binary(Vec<u8>),
    Closed {
        code: Option<u32>,
        reason: Option<String>,
        error: Option<String>,
    },
}

struct ConnectionState {
    write_half: SplitSink<WebSocketStream<TcpStream>, Message>,
    is_closed: bool,
    shutdown_tx: Option<oneshot::Sender<()>>,
}

pub struct WsManager {
    connections: Arc<Mutex<HashMap<usize, Arc<AsyncMutex<ConnectionState>>>>>,
    next_id: AtomicUsize,
    allowed_hosts: Option<Vec<String>>,
    package_name: String,
    sink: Arc<dyn WsEventSink>,
}

impl WsManager {
    #[tracing::instrument(level = "debug", skip_all)]
    pub(crate) fn new(
        allowed_hosts: Option<Vec<String>>,
        package_name: impl Into<String>,
        context: Weak<AsyncContext>,
    ) -> Self {
        Self::with_sink(
            allowed_hosts,
            package_name,
            Arc::new(ContextWsEventSink { context }),
        )
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn with_sink(
        allowed_hosts: Option<Vec<String>>,
        package_name: impl Into<String>,
        sink: Arc<dyn WsEventSink>,
    ) -> Self {
        Self {
            connections: Arc::new(Mutex::new(HashMap::new())),
            next_id: AtomicUsize::new(1),
            allowed_hosts,
            package_name: package_name.into(),
            sink,
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
        let (write_half, read_half) = socket.split();
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        let (event_tx, event_rx) = mpsc::unbounded_channel();

        let connection_id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let state = Arc::new(AsyncMutex::new(ConnectionState {
            write_half,
            is_closed: false,
            shutdown_tx: Some(shutdown_tx),
        }));

        {
            let mut connections = self.connections.lock().unwrap();
            connections.insert(connection_id, state.clone());
        }

        tokio::spawn(run_reader_loop(read_half, state, shutdown_rx, event_tx).in_current_span());
        tokio::spawn(
            run_dispatcher_loop(
                connection_id,
                self.package_name.clone(),
                self.connections.clone(),
                self.sink.clone(),
                event_rx,
            )
            .in_current_span(),
        );

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
        state.write_half.send(message).await?;
        Ok(WsSendResponse { success: true })
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
            if let Some(shutdown_tx) = state.shutdown_tx.take() {
                let _ = shutdown_tx.send(());
            }
            if !state.is_closed {
                state.is_closed = true;
                let close_frame = CloseFrame {
                    code: CloseCode::from(code),
                    reason: request.reason.map(|r| r.into()).unwrap_or_default(),
                };
                if let Err(err) = state
                    .write_half
                    .send(Message::Close(Some(close_frame)))
                    .await
                {
                    tracing::error!("Failed to send WebSocket close frame: {:?}", err);
                }
                if let Err(err) = state.write_half.flush().await {
                    tracing::error!("Failed to flush WebSocket close frame: {:?}", err);
                }
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
                        if let Some(shutdown_tx) = state.shutdown_tx.take() {
                            let _ = shutdown_tx.send(());
                        }
                        if !state.is_closed {
                            state.is_closed = true;
                            let _ = state.write_half.send(Message::Close(None)).await;
                            let _ = state.write_half.flush().await;
                        }
                    }
                }
                .in_current_span(),
            );
        }
    }
}

#[tracing::instrument(level = "debug", skip_all)]
async fn run_reader_loop(
    mut read_half: SplitStream<WebSocketStream<TcpStream>>,
    state: Arc<AsyncMutex<ConnectionState>>,
    mut shutdown_rx: oneshot::Receiver<()>,
    event_tx: mpsc::UnboundedSender<WsIncomingEvent>,
) {
    loop {
        tokio::select! {
            _ = &mut shutdown_rx => {
                break;
            }
            next_item = read_half.next() => {
                match next_item {
                    Some(Ok(message)) => {
                        if !handle_incoming_message(message, &state, &event_tx).await {
                            break;
                        }
                    }
                    Some(Err(err)) => {
                        let _ = event_tx.send(WsIncomingEvent::Closed {
                            code: None,
                            reason: None,
                            error: Some(err.to_string()),
                        });
                        break;
                    }
                    None => {
                        let _ = event_tx.send(WsIncomingEvent::Closed {
                            code: None,
                            reason: None,
                            error: None,
                        });
                        break;
                    }
                }
            }
        }
    }
}

#[tracing::instrument(level = "debug", skip_all)]
async fn handle_incoming_message(
    message: Message,
    state: &Arc<AsyncMutex<ConnectionState>>,
    event_tx: &mpsc::UnboundedSender<WsIncomingEvent>,
) -> bool {
    match message {
        Message::Text(text) => event_tx
            .send(WsIncomingEvent::Text(text.to_string()))
            .is_ok(),
        Message::Binary(bin) => event_tx.send(WsIncomingEvent::Binary(bin.to_vec())).is_ok(),
        Message::Ping(data) => {
            let mut guard = state.lock().await;
            if !guard.is_closed
                && let Err(err) = guard.write_half.send(Message::Pong(data)).await
            {
                tracing::error!("Failed to send WebSocket pong frame: {:?}", err);
            }
            true
        }
        Message::Close(frame) => {
            let (code, reason) = match frame {
                Some(close_frame) => (
                    Some(u16::from(close_frame.code) as u32),
                    Some(close_frame.reason.to_string()),
                ),
                None => (None, None),
            };
            let _ = event_tx.send(WsIncomingEvent::Closed {
                code,
                reason,
                error: None,
            });
            false
        }
        Message::Pong(_) | Message::Frame(_) => true,
    }
}

#[tracing::instrument(level = "debug", skip_all)]
async fn run_dispatcher_loop(
    connection_id: usize,
    package_name: String,
    connections: Arc<Mutex<HashMap<usize, Arc<AsyncMutex<ConnectionState>>>>>,
    sink: Arc<dyn WsEventSink>,
    mut event_rx: mpsc::UnboundedReceiver<WsIncomingEvent>,
) {
    while let Some(incoming) = event_rx.recv().await {
        let is_closed = matches!(incoming, WsIncomingEvent::Closed { .. });
        if is_closed {
            let mut connections_guard = connections.lock().unwrap();
            connections_guard.remove(&connection_id);
        }

        let event = match incoming {
            WsIncomingEvent::Text(text) => ws_event_request::Event::Text(text),
            WsIncomingEvent::Binary(binary) => ws_event_request::Event::Binary(binary),
            WsIncomingEvent::Closed {
                code,
                reason,
                error,
            } => ws_event_request::Event::Closed(WsClosed {
                code,
                reason,
                error,
            }),
        };

        let command = ExtensionCommand {
            package_name: package_name.clone(),
            event: Some(extension_command::Event::WsEvent(WsEventRequest {
                connection_id: connection_id as u64,
                event: Some(event),
            })),
        };

        if let Err(err) = sink.dispatch(command).await {
            tracing::error!(
                "Failed to dispatch WebSocket event for {}: {:?}",
                package_name,
                err
            );
            break;
        }
    }
}
