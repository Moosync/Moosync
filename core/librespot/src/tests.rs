use super::LibrespotHolder;

#[test]
fn test_librespot_holder_new() {
    let holder = LibrespotHolder::new();
    // is_initialized returns Ok(false) when not initialized but check didn't error
    let initialized = holder.is_initialized().unwrap();
    assert!(!initialized);
}

#[test]
fn test_register_event() {
    let holder = LibrespotHolder::new();
    let res = holder.register_event("test_event".to_string());
    assert!(res.is_ok());

    let events = super::REGISTERED_EVENTS.lock().unwrap();
    assert!(events.contains(&"test_event".to_string()));
}
