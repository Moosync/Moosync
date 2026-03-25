// PulseAudio sample rate monitoring (Linux only)
//
// This module monitors the system's audio configuration via PulseAudio/PipeWire:
// - Detects the default audio sink's sample rate on startup
// - Subscribes to sink/server changes to detect sample rate changes at runtime
//
// Architecture:
// - `start_pulse_monitor()` spawns a background thread running `run_pulse_monitor()`
// - The monitor connects to PulseAudio, queries initial state, then enters an event loop
// - Events (sample rate changes, sink changes) are sent via mpsc channel to the caller
//
// Why Rc<RefCell<T>>?
// - PulseAudio uses a callback-based async API
// - Callbacks must be 'static and need to access shared state (context, current rate, etc.)
// - Rc provides shared ownership, RefCell provides interior mutability

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use libpulse_binding as pulse;
use pulse::callbacks::ListResult;
use pulse::context::subscribe::{Facility, InterestMaskSet, Operation};
use pulse::context::{Context, FlagSet as ContextFlagSet, State as ContextState};
use pulse::mainloop::standard::{IterateResult, Mainloop};
use pulse::proplist::Proplist;
use thiserror::Error;
use tracing::trace;

/// Errors that can occur during PulseAudio operations
#[derive(Error, Debug, Clone)]
pub enum PulseError {
    #[error("Failed to create PulseAudio mainloop")]
    MainloopCreate,
    #[error("Failed to create proplist")]
    ProplistCreate,
    #[error("Failed to set application name")]
    SetAppName,
    #[error("Failed to create PulseAudio context")]
    ContextCreate,
    #[error("Failed to connect to PulseAudio: {0}")]
    Connect(String),
    #[error("Mainloop iteration failed")]
    MainloopIteration,
    #[error("Context connection failed")]
    ContextConnection,
    #[error("Mainloop error: {0}")]
    MainloopError(String),
    #[error("PulseAudio connection lost")]
    ConnectionLost,
    #[error("Failed to get sample rate")]
    SampleRateQuery,
}

/// Events sent from the PulseAudio monitor to the caller
#[derive(Debug, Clone)]
pub enum PulseEvent {
    /// Initial sample rate detected during startup
    SampleRateDetected(u32),
    /// Sample rate changed (e.g., user switched audio devices)
    SampleRateChanged { old: u32, new: u32 },
    /// Default sink changed (informational)
    DefaultSinkChanged(String),
    /// An error occurred in the monitor
    Error(PulseError),
}

/// Shared state for tracking the current audio configuration.
/// Used by callbacks to detect changes and send events.
struct MonitorState {
    /// Current default sink name (e.g., "alsa_output.pci-0000_00_1f.3.analog-stereo")
    sink_name: Rc<RefCell<Option<String>>>,
    /// Current sample rate in Hz (e.g., 48000)
    sample_rate: Rc<RefCell<Option<u32>>>,
}

impl MonitorState {
    fn new() -> Self {
        Self {
            sink_name: Rc::new(RefCell::new(None)),
            sample_rate: Rc::new(RefCell::new(None)),
        }
    }
}

// ============================================================================
// Public API
// ============================================================================

/// Start the PulseAudio monitor in a background thread.
///
/// Returns a channel receiver for PulseEvent notifications. The monitor will:
/// 1. Connect to PulseAudio/PipeWire
/// 2. Query the initial default sink and sample rate
/// 3. Subscribe to changes and send events when the sample rate changes
///
/// The monitor runs until PulseAudio disconnects or an error occurs.
pub fn start_pulse_monitor() -> Receiver<PulseEvent> {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        if let Err(e) = run_pulse_monitor(tx.clone()) {
            let _ = tx.send(PulseEvent::Error(e));
        }
    });

    rx
}

/// Synchronously get the default sink's sample rate.
///
/// This is a one-shot query that spawns a temporary thread, connects to PulseAudio,
/// queries the sample rate, and returns. Used for initial setup before the full
/// monitor is started.
///
/// Returns None if PulseAudio is not available or the query times out (2 seconds).
pub fn get_default_sample_rate() -> Option<u32> {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        if let Ok(rate) = get_sample_rate_sync() {
            let _ = tx.send(rate);
        }
    });

    rx.recv_timeout(std::time::Duration::from_secs(2)).ok()
}

// ============================================================================
// Monitor Implementation
// ============================================================================

/// Main monitor function - runs in a background thread.
///
/// Flow:
/// 1. Create PulseAudio mainloop and context
/// 2. Connect to server and wait for ready state
/// 3. Query initial default sink and sample rate
/// 4. Set up subscription callback for ongoing monitoring
/// 5. Run event loop until disconnection
fn run_pulse_monitor(tx: Sender<PulseEvent>) -> Result<(), PulseError> {
    // Step 1: Create mainloop and connect to PulseAudio
    let (mut mainloop, context) = create_pulse_connection()?;

    // Step 2: Wait for the connection to be ready
    wait_for_context_ready(&mut mainloop, &context)?;
    trace!("PulseAudio: Connected to server");

    // Step 3: Query initial state (default sink and its sample rate)
    let state = MonitorState::new();
    query_initial_state(&mut mainloop, &context, &state, &tx)?;

    // Step 4: Set up subscription for ongoing changes
    setup_subscribe_callback(&context, &state, &tx);

    // Step 5: Run the event loop
    run_event_loop(&mut mainloop, &context)
}

/// Create PulseAudio mainloop and context.
///
/// The context is wrapped in Rc<RefCell<>> because callbacks need to access it
/// for making introspect queries, and callbacks must be 'static.
fn create_pulse_connection() -> Result<(Mainloop, Rc<RefCell<Context>>), PulseError> {
    let mainloop = Mainloop::new().ok_or(PulseError::MainloopCreate)?;

    // Set up application properties for PulseAudio
    let mut proplist = Proplist::new().ok_or(PulseError::ProplistCreate)?;
    proplist
        .set_str(pulse::proplist::properties::APPLICATION_NAME, "Moosync")
        .map_err(|_| PulseError::SetAppName)?;

    let context = Context::new_with_proplist(&mainloop, "MoosyncPulseMonitor", &proplist)
        .ok_or(PulseError::ContextCreate)?;
    let context = Rc::new(RefCell::new(context));

    // Initiate connection (async - we'll wait for ready state next)
    context
        .borrow_mut()
        .connect(None, ContextFlagSet::NOFLAGS, None)
        .map_err(|e| PulseError::Connect(format!("{:?}", e)))?;

    Ok((mainloop, context))
}

/// Wait for the PulseAudio context to reach the Ready state.
///
/// PulseAudio connection is async - after calling connect(), we must iterate
/// the mainloop until the context state becomes Ready (or fails).
fn wait_for_context_ready(
    mainloop: &mut Mainloop,
    context: &Rc<RefCell<Context>>,
) -> Result<(), PulseError> {
    loop {
        match mainloop.iterate(true) {
            IterateResult::Quit(_) | IterateResult::Err(_) => {
                return Err(PulseError::MainloopIteration);
            }
            IterateResult::Success(_) => {}
        }

        match context.borrow().get_state() {
            ContextState::Ready => return Ok(()),
            ContextState::Failed | ContextState::Terminated => {
                return Err(PulseError::ContextConnection);
            }
            // Connecting, Authorizing, etc. - keep waiting
            _ => {}
        }
    }
}

/// Query the initial default sink and its sample rate.
///
/// This uses nested callbacks:
/// 1. get_server_info() -> returns default sink name
/// 2. get_sink_info_by_name() -> returns sink details including sample rate
///
/// After registering the callbacks, we iterate the mainloop to process them.
fn query_initial_state(
    mainloop: &mut Mainloop,
    context: &Rc<RefCell<Context>>,
    state: &MonitorState,
    tx: &Sender<PulseEvent>,
) -> Result<(), PulseError> {
    // Clone references for the callback (must be 'static)
    let tx_clone = tx.clone();
    let context_clone = Rc::clone(context);
    let sink_name_ref = Rc::clone(&state.sink_name);
    let sample_rate_ref = Rc::clone(&state.sample_rate);

    // Register callback to get server info (includes default sink name)
    let introspect = context.borrow().introspect();
    introspect.get_server_info(move |info| {
        if let Some(default_sink) = &info.default_sink_name {
            let sink_name = default_sink.to_string();
            trace!("PulseAudio: Default sink: {}", sink_name);
            *sink_name_ref.borrow_mut() = Some(sink_name.clone());

            // Nested callback: get sink details for sample rate
            let tx = tx_clone.clone();
            let rate_ref = Rc::clone(&sample_rate_ref);
            let introspect = context_clone.borrow().introspect();
            introspect.get_sink_info_by_name(&sink_name, move |result| {
                if let ListResult::Item(sink_info) = result {
                    let rate = sink_info.sample_spec.rate;
                    trace!("PulseAudio: Detected sample rate: {} Hz", rate);
                    *rate_ref.borrow_mut() = Some(rate);
                    let _ = tx.send(PulseEvent::SampleRateDetected(rate));
                }
            });
        }
    });

    // Iterate mainloop to process the callbacks (with timeout)
    for _ in 0..10 {
        if let IterateResult::Err(_) = mainloop.iterate(true) {
            return Err(PulseError::MainloopIteration);
        }
        if state.sample_rate.borrow().is_some() {
            break;
        }
    }

    Ok(())
}

/// Set up the subscription callback for ongoing monitoring.
///
/// PulseAudio will call this callback whenever:
/// - Server configuration changes (Facility::Server) - e.g., default sink changed
/// - Sink properties change (Facility::Sink) - e.g., sample rate changed
///
/// The callback queries current state and sends events if changes are detected.
fn setup_subscribe_callback(
    context: &Rc<RefCell<Context>>,
    state: &MonitorState,
    tx: &Sender<PulseEvent>,
) {
    let tx_subscribe = tx.clone();
    let context_subscribe = Rc::clone(context);
    let rate_ref = Rc::clone(&state.sample_rate);
    let sink_ref = Rc::clone(&state.sink_name);

    context.borrow_mut().set_subscribe_callback(Some(Box::new(
        move |facility, operation, index| {
            handle_subscribe_event(
                facility,
                operation,
                index,
                &tx_subscribe,
                &context_subscribe,
                &rate_ref,
                &sink_ref,
            );
        },
    )));

    // Tell PulseAudio which events we want to receive
    context.borrow_mut().subscribe(
        InterestMaskSet::SERVER | InterestMaskSet::SINK,
        |_success| {},
    );
}

/// Handle a subscription event from PulseAudio.
///
/// Called by PulseAudio when server or sink configuration changes.
fn handle_subscribe_event(
    facility: Option<Facility>,
    operation: Option<Operation>,
    index: u32,
    tx: &Sender<PulseEvent>,
    context: &Rc<RefCell<Context>>,
    rate_ref: &Rc<RefCell<Option<u32>>>,
    sink_ref: &Rc<RefCell<Option<String>>>,
) {
    match facility {
        Some(Facility::Server) => {
            // Server info changed - check if default sink changed
            handle_server_change(tx, context, rate_ref, sink_ref);
        }
        Some(Facility::Sink) if operation == Some(Operation::Changed) => {
            // Sink properties changed - check if it's our default sink
            handle_sink_change(index, tx, context, rate_ref, sink_ref);
        }
        _ => {}
    }
}

/// Handle a server change event (e.g., default sink switched).
fn handle_server_change(
    tx: &Sender<PulseEvent>,
    context: &Rc<RefCell<Context>>,
    rate_ref: &Rc<RefCell<Option<u32>>>,
    sink_ref: &Rc<RefCell<Option<String>>>,
) {
    let introspect = context.borrow().introspect();
    let tx = tx.clone();
    let sink_ref = sink_ref.clone();
    let rate_ref = rate_ref.clone();
    let context = Rc::clone(context);

    introspect.get_server_info(move |info| {
        if let Some(default_sink) = &info.default_sink_name {
            let new_sink = default_sink.to_string();
            let old_sink = sink_ref.borrow().clone();

            // Only process if the default sink actually changed
            if old_sink.as_ref() != Some(&new_sink) {
                trace!("PulseAudio: Default sink changed to: {}", new_sink);
                *sink_ref.borrow_mut() = Some(new_sink.clone());
                let _ = tx.send(PulseEvent::DefaultSinkChanged(new_sink.clone()));

                // Query the new sink's sample rate
                query_sink_sample_rate(&new_sink, &tx, &context, &rate_ref);
            }
        }
    });
}

/// Handle a sink change event (e.g., sample rate changed on current sink).
fn handle_sink_change(
    index: u32,
    tx: &Sender<PulseEvent>,
    context: &Rc<RefCell<Context>>,
    rate_ref: &Rc<RefCell<Option<u32>>>,
    sink_ref: &Rc<RefCell<Option<String>>>,
) {
    let introspect = context.borrow().introspect();
    let tx = tx.clone();
    let rate_ref = rate_ref.clone();
    let sink_ref = sink_ref.clone();

    introspect.get_sink_info_by_index(index, move |result| {
        if let ListResult::Item(sink_info) = result {
            let sink_name = sink_info
                .name
                .as_ref()
                .map(|s| s.to_string())
                .unwrap_or_default();
            let current_sink = sink_ref.borrow().clone();

            // Only process if this is our currently tracked sink
            if current_sink.as_ref() == Some(&sink_name) {
                check_and_send_rate_change(sink_info.sample_spec.rate, &tx, &rate_ref);
            }
        }
    });
}

/// Query a sink's sample rate by name and send event if changed.
fn query_sink_sample_rate(
    sink_name: &str,
    tx: &Sender<PulseEvent>,
    context: &Rc<RefCell<Context>>,
    rate_ref: &Rc<RefCell<Option<u32>>>,
) {
    let tx = tx.clone();
    let rate_ref = rate_ref.clone();
    let introspect = context.borrow().introspect();

    introspect.get_sink_info_by_name(sink_name, move |result| {
        if let ListResult::Item(sink_info) = result {
            check_and_send_rate_change(sink_info.sample_spec.rate, &tx, &rate_ref);
        }
    });
}

/// Check if sample rate changed and send event if so.
fn check_and_send_rate_change(
    new_rate: u32,
    tx: &Sender<PulseEvent>,
    rate_ref: &Rc<RefCell<Option<u32>>>,
) {
    let old_rate = *rate_ref.borrow();
    if old_rate != Some(new_rate) {
        trace!("PulseAudio: Sample rate changed: {:?} -> {}", old_rate, new_rate);
        *rate_ref.borrow_mut() = Some(new_rate);
        let _ = tx.send(PulseEvent::SampleRateChanged {
            old: old_rate.unwrap_or(0),
            new: new_rate,
        });
    }
}

/// Run the main event loop.
///
/// Iterates the PulseAudio mainloop, which:
/// - Processes incoming events and invokes our callbacks
/// - Handles PulseAudio protocol communication
///
/// Returns when the mainloop quits or the connection is lost.
fn run_event_loop(
    mainloop: &mut Mainloop,
    context: &Rc<RefCell<Context>>,
) -> Result<(), PulseError> {
    loop {
        match mainloop.iterate(true) {
            IterateResult::Quit(_) => {
                trace!("PulseAudio: Mainloop quit");
                return Ok(());
            }
            IterateResult::Err(e) => {
                return Err(PulseError::MainloopError(format!("{:?}", e)));
            }
            IterateResult::Success(_) => {}
        }

        // Check if context is still connected
        match context.borrow().get_state() {
            ContextState::Failed | ContextState::Terminated => {
                return Err(PulseError::ConnectionLost);
            }
            _ => {}
        }
    }
}

// ============================================================================
// One-shot Sample Rate Query
// ============================================================================

/// Synchronous sample rate query implementation.
///
/// Uses a two-phase approach to avoid nested callbacks:
/// 1. Query server info to get default sink name
/// 2. Query sink info to get sample rate
fn get_sample_rate_sync() -> Result<u32, PulseError> {
    let mut mainloop = Mainloop::new().ok_or(PulseError::MainloopCreate)?;
    let mut context = Context::new(&mainloop, "MoosyncSampleRateQuery")
        .ok_or(PulseError::ContextCreate)?;

    context
        .connect(None, ContextFlagSet::NOFLAGS, None)
        .map_err(|e| PulseError::Connect(format!("{:?}", e)))?;

    // Wait for connection
    wait_for_context_ready_simple(&mut mainloop, &mut context)?;

    // Phase 1: Get default sink name
    let sink_name = query_default_sink_name(&mut mainloop, &context)?;

    // Phase 2: Get sink sample rate
    query_sink_rate(&mut mainloop, &context, &sink_name)
}

/// Simplified wait for context ready (no Rc wrapper).
fn wait_for_context_ready_simple(
    mainloop: &mut Mainloop,
    context: &mut Context,
) -> Result<(), PulseError> {
    loop {
        if let IterateResult::Err(_) = mainloop.iterate(true) {
            return Err(PulseError::MainloopIteration);
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

/// Query the default sink name from server info.
fn query_default_sink_name(mainloop: &mut Mainloop, context: &Context) -> Result<String, PulseError> {
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

    // Process callback
    for _ in 0..10 {
        if let IterateResult::Err(_) = mainloop.iterate(true) {
            break;
        }
        if sink_name.borrow().is_some() {
            break;
        }
    }

    sink_name.borrow().clone().ok_or(PulseError::SampleRateQuery)
}

/// Query a sink's sample rate by name.
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

    // Process callback
    for _ in 0..10 {
        if let IterateResult::Err(_) = mainloop.iterate(true) {
            break;
        }
        if sample_rate.borrow().is_some() {
            break;
        }
    }

    sample_rate.borrow().ok_or(PulseError::SampleRateQuery)
}
