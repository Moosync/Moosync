// PulseAudio sample rate query (Linux only)
// Falls back to cpal on other platforms — cpal's default config has known bugs
// on PipeWire/Snapcast sinks, so we prefer the PulseAudio native protocol.

use std::{cell::RefCell, rc::Rc, sync::mpsc, thread, time::Duration};

use libpulse_binding as pulse;
use pulse::{
    callbacks::ListResult,
    context::{Context, FlagSet as ContextFlagSet, State as ContextState},
    mainloop::standard::{IterateResult, Mainloop},
};
use tracing::trace;

/// Synchronously query the default sink's sample rate from PulseAudio/PipeWire.
///
/// Returns `None` if PulseAudio is not running or the query times out
/// (2 second budget). The query spawns a short-lived thread that connects,
/// asks for the default sink + sample rate, then exits.
#[tracing::instrument(level = "debug", skip_all)]
pub fn get_default_sample_rate() -> Option<u32> {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let Ok(rate) = fetch_sample_rate() else {
            return;
        };
        let _ = tx.send(rate);
    });

    rx.recv_timeout(Duration::from_secs(2)).ok()
}

/// Synchronous two-phase sample rate query:
/// 1. server info → default sink name
/// 2. sink info → sample rate
#[tracing::instrument(level = "debug", skip_all)]
fn fetch_sample_rate() -> Result<u32, PulseError> {
    let mut mainloop = Mainloop::new().ok_or(PulseError::MainloopCreate)?;
    let mut context =
        Context::new(&mainloop, "MoosyncSampleRateQuery").ok_or(PulseError::ContextCreate)?;

    context
        .connect(None, ContextFlagSet::NOFLAGS, None)
        .map_err(|e| PulseError::Connect(format!("{:?}", e)))?;

    wait_for_ready(&mut mainloop, &mut context)?;
    trace!("PulseAudio: Connected to server");

    let sink_name = fetch_default_sink(&mut mainloop, &context)?;
    fetch_sink_rate(&mut mainloop, &context, &sink_name)
}

#[tracing::instrument(level = "debug", skip_all)]
fn step_mainloop(mainloop: &mut Mainloop) -> Result<(), PulseError> {
    let result = mainloop.iterate(true);
    let IterateResult::Success(_) = result else {
        return Err(PulseError::MainloopIteration);
    };
    Ok(())
}

#[tracing::instrument(level = "debug", skip_all)]
fn check_context_state(context: &Context) -> Result<bool, PulseError> {
    let state = context.get_state();
    if matches!(state, ContextState::Ready) {
        return Ok(true);
    }
    if matches!(state, ContextState::Failed | ContextState::Terminated) {
        return Err(PulseError::ContextConnection);
    }
    Ok(false)
}

#[tracing::instrument(level = "debug", skip_all)]
fn wait_for_ready(mainloop: &mut Mainloop, context: &mut Context) -> Result<(), PulseError> {
    loop {
        step_mainloop(mainloop)?;
        let is_ready = check_context_state(context)?;
        if is_ready {
            return Ok(());
        }
    }
}

#[tracing::instrument(level = "debug", skip_all)]
fn pump_until<T>(
    mainloop: &mut Mainloop,
    max_ticks: usize,
    check: impl Fn() -> Option<T>,
) -> Result<T, PulseError> {
    for _ in 0..max_ticks {
        step_mainloop(mainloop)?;
        let val = check();
        if let Some(res) = val {
            return Ok(res);
        }
    }
    Err(PulseError::SampleRateQuery)
}

#[tracing::instrument(level = "debug", skip_all)]
fn fetch_default_sink(mainloop: &mut Mainloop, context: &Context) -> Result<String, PulseError> {
    let sink_name: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));
    let sink_name_clone = Rc::clone(&sink_name);
    let introspect = context.introspect();
    introspect.get_server_info(move |info| {
        if let Some(name) = &info.default_sink_name {
            *sink_name_clone.borrow_mut() = Some(name.to_string());
        }
    });

    pump_until(mainloop, 10, || (*sink_name.borrow()).clone())
}

#[tracing::instrument(level = "debug", skip_all)]
fn fetch_sink_rate(
    mainloop: &mut Mainloop,
    context: &Context,
    sink_name: &str,
) -> Result<u32, PulseError> {
    let sample_rate: Rc<RefCell<Option<u32>>> = Rc::new(RefCell::new(None));
    let rate_clone = Rc::clone(&sample_rate);
    let introspect = context.introspect();
    introspect.get_sink_info_by_name(sink_name, move |list_result| {
        let ListResult::Item(sink_info) = list_result else {
            return;
        };
        *rate_clone.borrow_mut() = Some(sink_info.sample_spec.rate);
    });

    pump_until(mainloop, 10, || *sample_rate.borrow())
}

#[derive(thiserror::Error, Debug, Clone)]
pub enum PulseError {
    #[error("Failed to create PulseAudio mainloop")]
    MainloopCreate,
    #[error("Failed to create PulseAudio context")]
    ContextCreate,
    #[error("Failed to connect to PulseAudio: {0}")]
    Connect(String),
    #[error("Mainloop iteration failed")]
    MainloopIteration,
    #[error("Context connection failed")]
    ContextConnection,
    #[error("Failed to get sample rate")]
    SampleRateQuery,
}
