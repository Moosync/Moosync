// PulseAudio sample rate query (Linux only)
// Falls back to cpal on other platforms — cpal's default config has known bugs
// on PipeWire/Snapcast sinks, so we prefer the PulseAudio native protocol.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::{self, Receiver};
use std::thread;

use libpulse_binding as pulse;
use pulse::callbacks::ListResult;
use pulse::context::{Context, FlagSet as ContextFlagSet, State as ContextState};
use pulse::mainloop::standard::{IterateResult, Mainloop};
use pulse::proplist::Proplist;
use tracing::trace;

/// Synchronously query the default sink's sample rate from PulseAudio/PipeWire.
///
/// Returns `None` if PulseAudio is not running or the query times out
/// (2 second budget). The query spawns a short-lived thread that connects,
/// asks for the default sink + sample rate, then exits.
pub fn get_default_sample_rate() -> Option<u32> {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        if let Ok(rate) = get_sample_rate_sync() {
            let _ = tx.send(rate);
        }
    });

    rx.recv_timeout(std::time::Duration::from_secs(2)).ok()
}

/// Synchronous two-phase sample rate query:
/// 1. server info → default sink name
/// 2. sink info → sample rate
fn get_sample_rate_sync() -> Result<u32, PulseError> {
    let mut mainloop = Mainloop::new().ok_or(PulseError::MainloopCreate)?;
    let mut context = Context::new(&mainloop, "MoosyncSampleRateQuery")
        .ok_or(PulseError::ContextCreate)?;

    context
        .connect(None, ContextFlagSet::NOFLAGS, None)
        .map_err(|e| PulseError::Connect(format!("{:?}", e)))?;

    wait_for_context_ready(&mut mainloop, &mut context)?;
    trace!("PulseAudio: Connected to server");

    let sink_name = query_default_sink_name(&mut mainloop, &context)?;
    query_sink_rate(&mut mainloop, &context, &sink_name)
}

fn wait_for_context_ready(
    mainloop: &mut Mainloop,
    context: &mut Context,
) -> Result<(), PulseError> {
    loop {
        match mainloop.iterate(true) {
            IterateResult::Quit(_) | IterateResult::Err(_) => {
                return Err(PulseError::MainloopIteration);
            }
            IterateResult::Success(_) => {}
        }
        match context.get_state() {
            ContextState::Ready => return Ok(()),
            ContextState::Failed | ContextState::Terminated => {
                return Err(PulseError::ContextConnection);
            }
            _ => {}
        }
    }
}

fn query_default_sink_name(
    mainloop: &mut Mainloop,
    context: &Context,
) -> Result<String, PulseError> {
    let sink_name: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));
    {
        let sink_name_clone = Rc::clone(&sink_name);
        let introspect = context.introspect();
        introspect.get_server_info(move |info| {
            if let Some(name) = &info.default_sink_name {
                *sink_name_clone.borrow_mut() = Some(name.to_string());
            }
        });
    }

    for _ in 0..10 {
        match mainloop.iterate(true) {
            IterateResult::Quit(_) | IterateResult::Err(_) => {
                return Err(PulseError::MainloopIteration);
            }
            IterateResult::Success(_) => {}
        }
        if sink_name.borrow().is_some() {
            break;
        }
    }

    sink_name
        .borrow()
        .clone()
        .ok_or(PulseError::SampleRateQuery)
}

fn query_sink_rate(
    mainloop: &mut Mainloop,
    context: &Context,
    sink_name: &str,
) -> Result<u32, PulseError> {
    let sample_rate: Rc<RefCell<Option<u32>>> = Rc::new(RefCell::new(None));
    {
        let rate_clone = Rc::clone(&sample_rate);
        let introspect = context.introspect();
        introspect.get_sink_info_by_name(sink_name, move |list_result| {
            if let ListResult::Item(sink_info) = list_result {
                *rate_clone.borrow_mut() = Some(sink_info.sample_spec.rate);
            }
        });
    }

    for _ in 0..10 {
        match mainloop.iterate(true) {
            IterateResult::Quit(_) | IterateResult::Err(_) => {
                return Err(PulseError::MainloopIteration);
            }
            IterateResult::Success(_) => {}
        }
        if sample_rate.borrow().is_some() {
            break;
        }
    }

    sample_rate.borrow().clone().ok_or(PulseError::SampleRateQuery)
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
