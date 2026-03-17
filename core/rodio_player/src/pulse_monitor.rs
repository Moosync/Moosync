// PulseAudio sample rate monitoring (Linux only)
//
// This module provides:
// - Detection of the default sink's actual sample rate
// - Subscription to sink/server changes to detect sample rate changes

use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use libpulse_binding as pulse;
use pulse::callbacks::ListResult;
use pulse::context::introspect::SinkInfo;
use pulse::context::subscribe::{Facility, InterestMaskSet, Operation};
use pulse::context::{Context, FlagSet as ContextFlagSet, State as ContextState};
use pulse::mainloop::standard::{IterateResult, Mainloop};
use pulse::proplist::Proplist;

/// Events sent from the PulseAudio monitor
#[derive(Debug, Clone)]
pub enum PulseEvent {
    /// Initial sample rate detected
    SampleRateDetected(u32),
    /// Sample rate changed (old_rate, new_rate)
    SampleRateChanged { old: u32, new: u32 },
    /// Default sink changed
    DefaultSinkChanged(String),
    /// Error occurred
    Error(String),
}

/// Start the PulseAudio monitor in a background thread.
/// Returns a receiver for PulseEvent notifications.
pub fn start_pulse_monitor() -> Receiver<PulseEvent> {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        if let Err(e) = run_pulse_monitor(tx.clone()) {
            let _ = tx.send(PulseEvent::Error(e));
        }
    });

    rx
}

fn run_pulse_monitor(tx: Sender<PulseEvent>) -> Result<(), String> {
    // Create mainloop
    let mainloop = Rc::new(RefCell::new(
        Mainloop::new().ok_or("Failed to create PulseAudio mainloop")?,
    ));

    // Create context
    let mut proplist = Proplist::new().ok_or("Failed to create proplist")?;
    proplist
        .set_str(pulse::proplist::properties::APPLICATION_NAME, "Moosync")
        .map_err(|_| "Failed to set application name")?;

    let context = Rc::new(RefCell::new(
        Context::new_with_proplist(mainloop.borrow().deref(), "MoosyncPulseMonitor", &proplist)
            .ok_or("Failed to create PulseAudio context")?,
    ));

    // Connect to server
    context
        .borrow_mut()
        .connect(None, ContextFlagSet::NOFLAGS, None)
        .map_err(|e| format!("Failed to connect to PulseAudio: {:?}", e))?;

    // Wait for context to be ready
    loop {
        match mainloop.borrow_mut().iterate(true) {
            IterateResult::Quit(_) | IterateResult::Err(_) => {
                return Err("Mainloop iteration failed".to_string());
            }
            IterateResult::Success(_) => {}
        }

        match context.borrow().get_state() {
            ContextState::Ready => break,
            ContextState::Failed | ContextState::Terminated => {
                return Err("Context connection failed".to_string());
            }
            _ => {}
        }
    }

    eprintln!(">>> PulseAudio: Connected to server");

    // State for tracking sample rate
    let current_sample_rate: Rc<RefCell<Option<u32>>> = Rc::new(RefCell::new(None));
    let current_sink_name: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));

    // Get initial server info to find default sink
    let tx_clone = tx.clone();
    let context_clone = Rc::clone(&context);
    let mainloop_clone = Rc::clone(&mainloop);
    let current_sample_rate_clone = Rc::clone(&current_sample_rate);
    let current_sink_name_clone = Rc::clone(&current_sink_name);

    {
        let introspect = context.borrow().introspect();
        introspect.get_server_info(move |info| {
            if let Some(default_sink) = &info.default_sink_name {
                let sink_name = default_sink.to_string();
                eprintln!(">>> PulseAudio: Default sink: {}", sink_name);
                *current_sink_name_clone.borrow_mut() = Some(sink_name.clone());

                // Get sink info for sample rate
                let tx = tx_clone.clone();
                let rate_ref = Rc::clone(&current_sample_rate_clone);
                let introspect = context_clone.borrow().introspect();
                introspect.get_sink_info_by_name(&sink_name, move |result| {
                    if let ListResult::Item(sink_info) = result {
                        let rate = sink_info.sample_spec.rate;
                        eprintln!(">>> PulseAudio: Detected sample rate: {} Hz", rate);
                        *rate_ref.borrow_mut() = Some(rate);
                        let _ = tx.send(PulseEvent::SampleRateDetected(rate));
                    }
                });
            }
        });
    }

    // Process callbacks to get initial info
    for _ in 0..10 {
        if let IterateResult::Err(_) = mainloop.borrow_mut().iterate(true) {
            return Err("Mainloop iteration failed".to_string());
        }
        if current_sample_rate.borrow().is_some() {
            break;
        }
    }

    // Subscribe to sink and server changes
    let tx_subscribe = tx.clone();
    let context_subscribe = Rc::clone(&context);
    let rate_ref = Rc::clone(&current_sample_rate);
    let sink_ref = Rc::clone(&current_sink_name);

    context.borrow_mut().set_subscribe_callback(Some(Box::new(
        move |facility, operation, index| {
            let tx = tx_subscribe.clone();
            let context = Rc::clone(&context_subscribe);
            let rate_ref = Rc::clone(&rate_ref);
            let sink_ref = Rc::clone(&sink_ref);

            match facility {
                Some(Facility::Server) => {
                    // Server info changed - check if default sink changed
                    let introspect = context.borrow().introspect();
                    let tx = tx.clone();
                    let sink_ref = sink_ref.clone();
                    let rate_ref = rate_ref.clone();
                    let context = Rc::clone(&context);

                    introspect.get_server_info(move |info| {
                        if let Some(default_sink) = &info.default_sink_name {
                            let new_sink = default_sink.to_string();
                            let old_sink = sink_ref.borrow().clone();

                            if old_sink.as_ref() != Some(&new_sink) {
                                eprintln!(">>> PulseAudio: Default sink changed to: {}", new_sink);
                                *sink_ref.borrow_mut() = Some(new_sink.clone());
                                let _ = tx.send(PulseEvent::DefaultSinkChanged(new_sink.clone()));

                                // Get new sink's sample rate
                                let tx = tx.clone();
                                let rate_ref = rate_ref.clone();
                                let introspect = context.borrow().introspect();
                                introspect.get_sink_info_by_name(&new_sink, move |result| {
                                    if let ListResult::Item(sink_info) = result {
                                        let new_rate = sink_info.sample_spec.rate;
                                        let old_rate = *rate_ref.borrow();
                                        if old_rate != Some(new_rate) {
                                            eprintln!(
                                                ">>> PulseAudio: Sample rate changed: {:?} -> {}",
                                                old_rate, new_rate
                                            );
                                            *rate_ref.borrow_mut() = Some(new_rate);
                                            let _ = tx.send(PulseEvent::SampleRateChanged {
                                                old: old_rate.unwrap_or(0),
                                                new: new_rate,
                                            });
                                        }
                                    }
                                });
                            }
                        }
                    });
                }
                Some(Facility::Sink) if operation == Some(Operation::Changed) => {
                    // Sink properties changed - check if it's our default sink
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

                            if current_sink.as_ref() == Some(&sink_name) {
                                let new_rate = sink_info.sample_spec.rate;
                                let old_rate = *rate_ref.borrow();
                                if old_rate != Some(new_rate) {
                                    eprintln!(
                                        ">>> PulseAudio: Sample rate changed: {:?} -> {}",
                                        old_rate, new_rate
                                    );
                                    *rate_ref.borrow_mut() = Some(new_rate);
                                    let _ = tx.send(PulseEvent::SampleRateChanged {
                                        old: old_rate.unwrap_or(0),
                                        new: new_rate,
                                    });
                                }
                            }
                        }
                    });
                }
                _ => {}
            }
        },
    )));

    // Subscribe to server and sink events
    context.borrow_mut().subscribe(
        InterestMaskSet::SERVER | InterestMaskSet::SINK,
        |_success| {},
    );

    // Run the mainloop
    loop {
        match mainloop.borrow_mut().iterate(true) {
            IterateResult::Quit(_) => {
                eprintln!(">>> PulseAudio: Mainloop quit");
                break;
            }
            IterateResult::Err(e) => {
                return Err(format!("Mainloop error: {:?}", e));
            }
            IterateResult::Success(_) => {}
        }

        // Check if context is still connected
        match context.borrow().get_state() {
            ContextState::Failed | ContextState::Terminated => {
                return Err("PulseAudio connection lost".to_string());
            }
            _ => {}
        }
    }

    Ok(())
}

/// Synchronously get the default sink's sample rate.
/// Returns None if PulseAudio is not available or an error occurs.
pub fn get_default_sample_rate() -> Option<u32> {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        if let Ok(rate) = get_sample_rate_sync() {
            let _ = tx.send(rate);
        }
    });

    // Wait up to 2 seconds for the result
    rx.recv_timeout(std::time::Duration::from_secs(2)).ok()
}

fn get_sample_rate_sync() -> Result<u32, String> {
    let mainloop = Mainloop::new().ok_or("Failed to create mainloop")?;
    let mainloop = Rc::new(RefCell::new(mainloop));

    let context = Context::new(mainloop.borrow().deref(), "MoosyncSampleRateQuery")
        .ok_or("Failed to create context")?;
    let context = Rc::new(RefCell::new(context));

    context
        .borrow_mut()
        .connect(None, ContextFlagSet::NOFLAGS, None)
        .map_err(|e| format!("Failed to connect: {:?}", e))?;

    // Wait for ready
    loop {
        if let IterateResult::Err(_) = mainloop.borrow_mut().iterate(true) {
            return Err("Iteration failed".to_string());
        }
        match context.borrow().get_state() {
            ContextState::Ready => break,
            ContextState::Failed | ContextState::Terminated => {
                return Err("Connection failed".to_string());
            }
            _ => {}
        }
    }

    let result: Rc<RefCell<Option<u32>>> = Rc::new(RefCell::new(None));
    let result_clone = Rc::clone(&result);
    let context_clone = Rc::clone(&context);

    {
        let introspect = context.borrow().introspect();
        introspect.get_server_info(move |info| {
            if let Some(sink_name) = &info.default_sink_name {
                let sink_name = sink_name.to_string();
                let result = Rc::clone(&result_clone);
                let introspect = context_clone.borrow().introspect();
                introspect.get_sink_info_by_name(&sink_name, move |list_result| {
                    if let ListResult::Item(sink_info) = list_result {
                        *result.borrow_mut() = Some(sink_info.sample_spec.rate);
                    }
                });
            }
        });
    }

    // Process callbacks
    for _ in 0..20 {
        if let IterateResult::Err(_) = mainloop.borrow_mut().iterate(true) {
            break;
        }
        if result.borrow().is_some() {
            break;
        }
    }

    result
        .borrow()
        .ok_or_else(|| "Failed to get sample rate".to_string())
}
