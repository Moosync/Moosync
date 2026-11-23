use std::cell::RefCell;

use rstest::rstest;
use tempdir::TempDir;
use tracing_test::traced_test;

use crate::{LogController, ProfilingMode, toggle_tracy_profiling_with};

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_toggle_tracy_enables_layer_without_launching() {
    let modes = RefCell::new(Vec::<&'static str>::new());

    let outcome = toggle_tracy_profiling_with(false, |mode| {
        let label = match mode {
            ProfilingMode::Disabled => "disabled",
            ProfilingMode::Tracy => "tracy",
        };
        modes.borrow_mut().push(label);
        Ok(())
    })
    .unwrap();

    assert!(outcome.enabled);
    assert_eq!(modes.borrow().as_slice(), &["tracy"]);
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_toggle_tracy_disables_when_already_enabled() {
    let modes = RefCell::new(Vec::<&'static str>::new());

    let outcome = toggle_tracy_profiling_with(true, |mode| {
        let label = match mode {
            ProfilingMode::Disabled => "disabled",
            ProfilingMode::Tracy => "tracy",
        };
        modes.borrow_mut().push(label);
        Ok(())
    })
    .unwrap();

    assert!(!outcome.enabled);
    assert_eq!(modes.borrow().as_slice(), &["disabled"]);
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_log_controller_returns_log_dir() {
    let temp_dir = TempDir::new("moosync_log_ctrl_test").unwrap();
    let log_dir = temp_dir.path().join("logs");
    let controller = LogController::new(log_dir.clone());

    assert_eq!(controller.get_log_dir(), log_dir.to_string_lossy());
    assert!(!controller.get_current_log_level().is_empty());
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_build_env_filter_string_combines_global_component_and_custom() {
    let filter = LogController::build_env_filter_string(
        "info",
        &[
            ("player", "debug"),
            ("database", "inherit"),
            ("extensions", "trace"),
        ],
        "hyper=warn",
    );
    assert_eq!(filter, "info,player=debug,extensions=trace,hyper=warn");
}

#[rstest]
#[tracing::instrument(level = "debug", skip_all)]
fn test_reload_layer_with_filter_receives_debug_spans_and_events_after_reload() {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    use tracing::Subscriber;
    use tracing_subscriber::{EnvFilter, Layer, Registry, layer::SubscriberExt, reload};

    struct CountingLayer {
        spans: Arc<AtomicUsize>,
        events: Arc<AtomicUsize>,
    }

    impl<S: Subscriber> Layer<S> for CountingLayer {
        fn on_new_span(
            &self,
            _attrs: &tracing::span::Attributes<'_>,
            _id: &tracing::span::Id,
            _ctx: tracing_subscriber::layer::Context<'_, S>,
        ) {
            self.spans.fetch_add(1, Ordering::Relaxed);
        }

        fn on_event(
            &self,
            _event: &tracing::Event<'_>,
            _ctx: tracing_subscriber::layer::Context<'_, S>,
        ) {
            self.events.fetch_add(1, Ordering::Relaxed);
        }
    }

    let (profiling_layer, profiling_handle) =
        reload::Layer::new(None::<Box<dyn Layer<Registry> + Send + Sync>>);
    let subscriber =
        tracing_subscriber::registry().with(profiling_layer.with_filter(EnvFilter::new("debug")));

    let spans = Arc::new(AtomicUsize::new(0));
    let events = Arc::new(AtomicUsize::new(0));

    tracing::subscriber::with_default(subscriber, || {
        let span_before = tracing::debug_span!("before_reload");
        let _g = span_before.enter();
        tracing::debug!("event_before");
        assert_eq!(spans.load(Ordering::Relaxed), 0);
        assert_eq!(events.load(Ordering::Relaxed), 0);

        let layer: Box<dyn Layer<Registry> + Send + Sync> = Box::new(CountingLayer {
            spans: spans.clone(),
            events: events.clone(),
        });
        profiling_handle.reload(Some(layer)).unwrap();

        let span_after = tracing::debug_span!("after_reload");
        let _g2 = span_after.enter();
        tracing::debug!("event_after");
    });

    assert_eq!(
        spans.load(Ordering::Relaxed),
        1,
        "expected span after reload"
    );
    assert_eq!(
        events.load(Ordering::Relaxed),
        1,
        "expected event after reload"
    );
}
