use assertables::assert_none;
use rstest::rstest;
use tracing_test::traced_test;

use super::{pulse_monitor, pulse_monitor::PulseError};

struct EnvVarGuard {
    key: &'static str,
    prev: Option<String>,
}

impl EnvVarGuard {
    #[tracing::instrument(level = "debug", skip_all)]
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
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_pulse_monitor_unreachable_server() {
    let temp_dir = std::env::temp_dir();
    let socket_path = temp_dir.join(format!("moosync_pulse_{}", std::process::id()));
    if socket_path.exists() {
        let _ = std::fs::remove_file(&socket_path);
    }
    let _guard = EnvVarGuard::set("PULSE_SERVER", &format!("unix:{}", socket_path.display()));

    let rate = pulse_monitor::get_default_sample_rate();

    assert_none!(rate);
}

#[test]
#[traced_test]
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
    let _ = handle.join();
    let _ = std::fs::remove_file(&temp_base);

    assert_none!(rate);
}

#[rstest]
#[case(PulseError::MainloopCreate, "Failed to create PulseAudio mainloop")]
#[case(PulseError::ContextCreate, "Failed to create PulseAudio context")]
#[case(PulseError::Connect("refused".into()), "Failed to connect to PulseAudio: refused")]
#[case(PulseError::Timeout, "PulseAudio connection timed out")]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_pulse_error_variants(#[case] err: PulseError, #[case] expected: &str) {
    let err_str = err.to_string();

    assert_eq!(err_str, expected);
}
