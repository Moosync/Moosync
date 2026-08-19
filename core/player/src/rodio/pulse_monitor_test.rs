use super::{pulse_monitor, pulse_monitor::PulseError};

struct EnvVarGuard {
    key: &'static str,
    prev: Option<String>,
}

impl EnvVarGuard {
    fn set(key: &'static str, val: &str) -> Self {
        let prev = std::env::var(key).ok();
        unsafe {
            std::env::set_var(key, val);
        }
        Self { key, prev }
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        unsafe {
            if let Some(ref val) = self.prev {
                std::env::set_var(self.key, val);
            } else {
                std::env::remove_var(self.key);
            }
        }
    }
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_pulse_monitor_unreachable_server() {
    let temp_dir = std::env::temp_dir();
    let socket_path = temp_dir.join(format!("moosync_pulse_{}", std::process::id()));
    if socket_path.exists() {
        let _ = std::fs::remove_file(&socket_path);
    }

    let _guard = EnvVarGuard::set("PULSE_SERVER", &format!("unix:{}", socket_path.display()));
    let rate = pulse_monitor::get_default_sample_rate();
    assert!(rate.is_none());
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_pulse_monitor_mock_dummy_unix_server() {
    use std::{env::temp_dir, os::unix::net::UnixListener, thread, time::Duration};

    let temp_base = temp_dir().join(format!("moosync_pulse_dummy_{}", std::process::id()));
    if temp_base.exists() {
        let _ = std::fs::remove_file(&temp_base);
    }

    let listener = UnixListener::bind(&temp_base).expect("Failed to bind mock Unix socket");
    listener.set_nonblocking(true).unwrap();

    let handle = thread::spawn(move || {
        for _ in 0..20 {
            if let Ok((stream, _)) = listener.accept() {
                drop(stream);
                break;
            }
            thread::sleep(Duration::from_millis(50));
        }
    });

    let _guard = EnvVarGuard::set("PULSE_SERVER", &format!("unix:{}", temp_base.display()));
    let rate = pulse_monitor::get_default_sample_rate();
    assert!(rate.is_none());

    let _ = handle.join();
    let _ = std::fs::remove_file(&temp_base);
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_pulse_error_variants() {
    let err1 = PulseError::MainloopCreate;
    assert_eq!(err1.to_string(), "Failed to create PulseAudio mainloop");

    let err2 = PulseError::ContextCreate;
    assert_eq!(err2.to_string(), "Failed to create PulseAudio context");

    let err3 = PulseError::Connect("refused".into());
    assert_eq!(err3.to_string(), "Failed to connect to PulseAudio: refused");

    let err4 = PulseError::Timeout;
    assert_eq!(err4.to_string(), "PulseAudio connection timed out");
}
