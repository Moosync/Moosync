# Dynamic Sample Rate Switching

## Overview

When the PulseAudio/PipeWire default device changes or its sample rate changes, the player should automatically reinitialize its audio stream to match the new sample rate.

## Current State

- `pulse_monitor.rs` already implements PulseAudio event subscription
- `start_pulse_monitor()` returns a receiver that emits:
  - `PulseEvent::SampleRateDetected(u32)` - initial detection
  - `PulseEvent::SampleRateChanged { old, new }` - rate changed
  - `PulseEvent::DefaultSinkChanged(String)` - default device changed
- Currently only `get_default_sample_rate()` is used for one-time detection at startup

## Implementation Plan

### 1. Add new command for stream reinitialization

```rust
enum RodioCommand {
    // ... existing commands ...
    ReinitStream(u32),  // New sample rate
}
```

### 2. Refactor stream creation

Extract stream creation into a separate function that can be called both at startup and when reinitializing:

```rust
fn create_audio_stream(sample_rate: u32) -> (MixerDeviceSink, Arc<rodio::Player>) {
    let stream_handle = rodio::DeviceSinkBuilder::from_default_device()
        .expect("No audio device found")
        .with_sample_rate(NonZero::new(sample_rate).unwrap())
        .open_stream()
        .unwrap();
    let sink = Arc::new(rodio::Player::connect_new(stream_handle.mixer()));
    (stream_handle, sink)
}
```

### 3. Start pulse monitor in player initialization

```rust
fn initialize(events_tx: Sender<PlayerEvent>) -> Sender<RodioCommand> {
    let (tx, rx) = channel::<RodioCommand>();

    // Start pulse monitor
    #[cfg(target_os = "linux")]
    let pulse_rx = pulse_monitor::start_pulse_monitor();

    // Forward sample rate changes to command channel
    #[cfg(target_os = "linux")]
    {
        let tx_clone = tx.clone();
        thread::spawn(move || {
            while let Ok(event) = pulse_rx.recv() {
                if let PulseEvent::SampleRateChanged { new, .. } = event {
                    let _ = tx_clone.send(RodioCommand::ReinitStream(new));
                }
            }
        });
    }

    // ... rest of initialization
}
```

### 4. Handle ReinitStream command

In the command loop:

```rust
RodioCommand::ReinitStream(new_rate) => {
    eprintln!(">>> Reinitializing stream with sample rate: {} Hz", new_rate);

    // Save current playback state
    let was_playing = !sink.is_paused();
    let current_src = last_src.lock().unwrap().clone();

    // Stop current playback
    sink.stop();
    sink.clear();

    // Drop old stream and create new one
    drop(stream_handle);
    let (new_stream_handle, new_sink) = create_audio_stream(new_rate);
    stream_handle = new_stream_handle;
    sink = new_sink;
    output_sample_rate = new_rate;

    // Resume playback if we were playing
    if let Some(src) = current_src {
        if was_playing {
            // TODO: Seek to previous position
            Self::set_src(src, &sink, output_sample_rate).await;
            sink.play();
        }
    }

    Self::send_event(events_tx.clone(), PlayerEvent::SampleRateChanged(new_rate));
}
```

### 5. Add PlayerEvent for sample rate changes

In the protobuf definitions, add a new event type so the UI can be notified:

```protobuf
message PlayerEvent {
    oneof event {
        // ... existing events ...
        uint32 sample_rate_changed = X;
    }
}
```

## Considerations

- **Seek position**: When reinitializing, we should try to resume from the same position. This requires tracking the current playback position.
- **Glitch-free transition**: There will likely be a brief audio gap during reinitialization. This is acceptable for device changes.
- **Debouncing**: Multiple rapid changes should be debounced to avoid excessive reinitialization.
- **Error handling**: If stream creation fails with the new rate, fall back to the previous rate or a safe default.

## Testing

1. Start playback
2. Change default PulseAudio/PipeWire device (e.g., switch from speakers to headphones)
3. Verify audio continues playing without manual intervention
4. Check debug output shows stream reinitialization

## Platform Notes

- Linux: Full support via libpulse-binding
- Windows/macOS: Not implemented - would need platform-specific APIs (WASAPI change notifications, CoreAudio property listeners)
