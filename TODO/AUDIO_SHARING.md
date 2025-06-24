# Audio Sharing

This document outlines the API for live-streaming audio files between devices using Moosync.

## Overview

Audio sharing allows users to stream their music playback to other devices in real-time, creating a synchronized listening experience across multiple Moosync instances.

## Architecture

### Components:

1. **Host Device**: The device sharing audio (source)
2. **Client Device(s)**: Device(s) receiving audio (receivers)
3. **Discovery Service**: Network discovery mechanism
4. **Streaming Protocol**: Real-time audio transmission
5. **Synchronization**: Ensures audio sync across devices

### Network Topology:

```
Host Device (Source)
    ├── Audio Encoder
    ├── Network Server
    └── Sync Controller
         ║
    ╔════╩════╗
    ║ Network ║
    ╚════╦════╝
         ║
Client Device(s) (Receivers)
    ├── Audio Decoder
    ├── Network Client
    ├── Buffer Management
    └── Playback Controller
```

## Discovery Protocol

### Service Advertisement (Host):

```json
{
  "service_type": "moosync-audio-share",
  "version": "1.0",
  "host_info": {
    "device_id": "uuid-v4",
    "device_name": "John's MacBook",
    "user_name": "John Doe",
    "avatar": "base64-encoded-image",
    "moosync_version": "2.1.0"
  },
  "stream_info": {
    "audio_format": "opus",
    "sample_rate": 48000,
    "channels": 2,
    "bitrate": 128000,
    "latency": "low"
  },
  "network": {
    "protocol": "tcp",
    "port": 8080,
    "encryption": "tls",
    "authentication": "password"
  },
  "capabilities": {
    "max_clients": 10,
    "queue_control": true,
    "chat": true,
    "metadata_sync": true
  }
}
```

### Service Discovery (Client):

```rust
// mDNS/Bonjour discovery
struct AudioShareService {
    device_id: String,
    device_name: String,
    address: IpAddr,
    port: u16,
    capabilities: ServiceCapabilities,
}

async fn discover_services() -> Result<Vec<AudioShareService>> {
    let mut services = Vec::new();
    let mdns = mdns::Responder::spawn(&tokio::runtime::Handle::current())?;
    
    let stream = mdns.service_resolver("_moosync-audio._tcp.local.")?;
    tokio::pin!(stream);
    
    while let Some(event) = stream.next().await {
        match event {
            ServiceEvent::ServiceResolved(info) => {
                services.push(parse_service_info(info)?);
            }
            _ => {}
        }
    }
    
    Ok(services)
}
```

## Authentication & Security

### Connection Handshake:

```json
{
  "connection_request": {
    "client_id": "uuid-v4",
    "client_name": "Sarah's iPhone",
    "protocol_version": "1.0",
    "authentication": {
      "method": "password",
      "credentials": "hashed-password"
    },
    "capabilities": {
      "audio_formats": ["opus", "aac"],
      "max_buffer_size": "5s",
      "low_latency": true
    }
  }
}
```

### Security Features:

- **TLS Encryption**: All communication encrypted with TLS 1.3
- **Device Authentication**: Password or certificate-based auth
- **Permission System**: Fine-grained control over client capabilities
- **Rate Limiting**: Prevent abuse and ensure quality

## Audio Streaming Protocol

### Stream Format:

```rust
struct AudioPacket {
    sequence_number: u64,
    timestamp: u64,          // Microseconds since stream start
    sample_rate: u32,
    channels: u8,
    codec: AudioCodec,
    data: Vec<u8>,           // Compressed audio data
    metadata: Option<PacketMetadata>,
}

struct PacketMetadata {
    song_id: Option<String>,
    position: Option<f64>,   // Current playback position
    volume: Option<f64>,     // Current volume level
}
```

### Codec Support:

1. **Opus** (Primary): Low latency, high quality
2. **AAC**: Apple ecosystem compatibility  
3. **FLAC**: Lossless for high-quality streaming
4. **MP3**: Fallback for compatibility

### Adaptive Bitrate:

```rust
struct BitrateController {
    target_bitrate: u32,
    current_bitrate: u32,
    network_quality: NetworkQuality,
    buffer_health: BufferHealth,
}

impl BitrateController {
    fn adjust_bitrate(&mut self, network_stats: &NetworkStats) {
        match (network_stats.packet_loss, network_stats.rtt) {
            (loss, _) if loss > 0.05 => self.decrease_bitrate(),
            (_, rtt) if rtt > Duration::from_millis(200) => self.decrease_bitrate(),
            _ if self.buffer_health.is_healthy() => self.increase_bitrate(),
            _ => {}
        }
    }
}
```

## Synchronization

### Time Synchronization:

```rust
struct SyncController {
    master_clock: SystemTime,
    offset: Duration,
    drift_compensation: f64,
}

// Network Time Protocol (NTP) style sync
async fn synchronize_clocks(host: &SocketAddr) -> Result<Duration> {
    let t1 = SystemTime::now();
    let sync_request = SyncRequest { client_time: t1 };
    
    let response = send_sync_request(host, sync_request).await?;
    let t4 = SystemTime::now();
    
    // Calculate offset: ((t2 - t1) + (t3 - t4)) / 2
    let offset = ((response.server_receive_time - t1) + 
                  (response.server_send_time - t4)) / 2;
    
    Ok(offset)
}
```

### Buffer Management:

```rust
struct AudioBuffer {
    target_size: Duration,      // Target buffer size (e.g., 2 seconds)
    current_size: Duration,     // Current buffer content
    packets: VecDeque<AudioPacket>,
    underrun_count: u32,
    overrun_count: u32,
}

impl AudioBuffer {
    fn should_play_now(&self, packet: &AudioPacket) -> bool {
        let current_time = self.get_synchronized_time();
        let packet_time = packet.timestamp;
        let buffer_delay = self.target_size;
        
        current_time + buffer_delay >= packet_time
    }
}
```

## API Implementation

### Host API:

```rust
// Start sharing audio
pub async fn start_audio_share(config: ShareConfig) -> Result<ShareSession> {
    let session = ShareSession::new(config)?;
    session.start_discovery().await?;
    session.start_audio_server().await?;
    Ok(session)
}

// Stop sharing
pub async fn stop_audio_share(session: ShareSession) -> Result<()> {
    session.stop_discovery().await?;
    session.stop_audio_server().await?;
    Ok(())
}

// Client management
pub fn get_connected_clients(session: &ShareSession) -> Vec<ClientInfo> {
    session.clients.lock().unwrap().clone()
}

pub async fn kick_client(session: &ShareSession, client_id: &str) -> Result<()> {
    session.disconnect_client(client_id).await
}
```

### Client API:

```rust
// Discover available streams
pub async fn discover_audio_streams() -> Result<Vec<AudioShareService>> {
    discover_services().await
}

// Connect to stream
pub async fn connect_to_stream(service: &AudioShareService, password: Option<&str>) 
    -> Result<StreamConnection> {
    let connection = StreamConnection::connect(service, password).await?;
    Ok(connection)
}

// Disconnect from stream  
pub async fn disconnect_from_stream(connection: StreamConnection) -> Result<()> {
    connection.disconnect().await
}
```

### Event System:

```rust
pub enum AudioShareEvent {
    // Host events
    ClientConnected { client_id: String, client_info: ClientInfo },
    ClientDisconnected { client_id: String },
    StreamStarted { session_id: String },
    StreamStopped { session_id: String },
    
    // Client events
    Connected { host_info: HostInfo },
    Disconnected { reason: DisconnectReason },
    AudioPacketReceived { packet: AudioPacket },
    BufferUnderrun,
    BufferOverrun,
    
    // Shared events
    NetworkError { error: NetworkError },
    SyncLost,
    QualityChanged { new_quality: StreamQuality },
}
```

## Configuration

### Host Configuration:

```json
{
  "sharing": {
    "enabled": true,
    "device_name": "Custom Device Name",
    "password_required": true,
    "password": "user-set-password",
    "max_clients": 5,
    "audio_quality": {
      "codec": "opus",
      "bitrate": 128000,
      "sample_rate": 48000,
      "adaptive_bitrate": true
    },
    "permissions": {
      "allow_queue_control": false,
      "allow_playback_control": false,
      "allow_chat": true
    }
  }
}
```

### Client Configuration:

```json
{
  "receiving": {
    "enabled": true,
    "auto_discover": true,
    "buffer_size": "2s",
    "audio_output": {
      "device_id": "default",
      "volume_sync": true,
      "independent_volume": false
    },
    "ui": {
      "show_host_metadata": true,
      "show_connection_status": true,
      "show_buffer_health": false
    }
  }
}
```

## Error Handling

### Common Error Types:

```rust
pub enum AudioShareError {
    NetworkError(NetworkError),
    AuthenticationFailed,
    UnsupportedCodec(String),
    BufferUnderrun,
    BufferOverrun,
    SyncLost,
    ClientLimitReached,
    InvalidPacket,
    ConnectionTimeout,
}
```

### Graceful Degradation:

- **Network Issues**: Reduce quality, increase buffering
- **Authentication**: Retry with user prompt
- **Codec Problems**: Fallback to supported codec
- **Buffer Issues**: Adjust buffer size dynamically
- **Sync Loss**: Re-synchronize clocks

## Performance Considerations

### Optimization Strategies:

1. **CPU Usage**: Hardware-accelerated encoding/decoding when available
2. **Memory Usage**: Circular buffers, packet pooling
3. **Network Usage**: Adaptive bitrate, compression
4. **Latency**: Low-latency codecs, optimized buffers
5. **Battery Life**: Power-aware encoding on mobile devices

### Monitoring:

```rust
struct StreamMetrics {
    bitrate: u32,
    packet_loss: f64,
    latency: Duration,
    buffer_health: f64,
    cpu_usage: f64,
    memory_usage: usize,
    network_throughput: u64,
}
```
