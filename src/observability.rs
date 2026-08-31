use std::{
    collections::HashMap,
    sync::Mutex,
    time::{Duration, Instant},
};

use tracing::{
    Subscriber,
    field::{Field, Visit},
    span,
};
use tracing_subscriber::{
    EnvFilter,
    fmt::{self, time::ChronoLocal},
    layer::{Layer, SubscriberExt},
    registry::LookupSpan,
    util::SubscriberInitExt,
};

/// Configure the global tracing subscriber as a layered registry:
///
/// - a `fmt` layer for human-readable logs, filtered by `RUST_LOG`
/// - a [`SlowSpanLayer`] that warns when a span overruns `SLOW_SPAN_MS`
/// - (with the `console` feature) a `console-subscriber` layer for tokio-console
pub fn init_tracing() {
    if std::env::var("RUST_LOG").is_err() {
        // SAFETY: called once at startup, before any other thread reads it.
        unsafe { std::env::set_var("RUST_LOG", "info") }
    }

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    // The filter applies to the fmt layer only, so the console layer still
    // receives tokio's internal trace events (it performs its own filtering).
    let fmt_layer = fmt::layer()
        .with_timer(ChronoLocal::new(String::from("%Y-%m-%d %I:%M:%S%.3f %p")))
        .with_target(true)
        .with_filter(env_filter);

    let registry = tracing_subscriber::registry()
        .with(fmt_layer)
        .with(SlowSpanLayer::from_env());

    #[cfg(feature = "console")]
    let registry = registry.with(console_subscriber::spawn());

    registry.init();
}

/// Emits a warning for any span whose lifetime exceeds `threshold`.
///
/// Override the default (100 ms) at runtime with the `SLOW_SPAN_MS`
/// environment variable.
///
/// Tokio's internal runtime spans and the per-actor task-root spans
/// ([`LONG_LIVED_SPAN_NAMES`]) are excluded, since both are expected to live
/// far longer than any reasonable threshold.
pub struct SlowSpanLayer {
    threshold: Duration,
    start_times: Mutex<HashMap<span::Id, Instant>>,
    fields: Mutex<HashMap<span::Id, Vec<(String, String)>>>,
}

impl SlowSpanLayer {
    pub fn new(threshold: Duration) -> Self {
        Self {
            threshold,
            start_times: Mutex::new(HashMap::new()),
            fields: Mutex::new(HashMap::new()),
        }
    }

    pub fn from_env() -> Self {
        let threshold = std::env::var("SLOW_SPAN_MS")
            .ok()
            .and_then(|value| value.parse::<u64>().ok())
            .map(Duration::from_millis)
            .unwrap_or_else(|| Duration::from_millis(100));
        Self::new(threshold)
    }
}

/// Names of the per-actor task-root spans (`#[tracing::instrument(name = "...")]`
/// on `run_world`/`run_map`/`run_db`) that wrap an actor's entire `rx.recv()`
/// loop for the lifetime of the process. Their "duration" is meaningless for
/// slow-span detection — they're expected to run for hours or days — so they're
/// excluded rather than reported as one giant slow span whenever the actor
/// finally shuts down.
const LONG_LIVED_SPAN_NAMES: &[&str] = &["world", "map", "db"];

/// Records span fields as plain `name=value` pairs.
///
/// The `fmt` layer's own `FormattedFields` extension bakes in ANSI color
/// codes (dim/italic styling for field names) whenever ANSI is enabled, which
/// then shows up as literal escape sequences in the slow-span warning
/// whenever that warning's output isn't a real terminal (a log file, `docker
/// logs`, journald, etc.). Collecting fields ourselves, independent of the
/// `fmt` layer's rendering, avoids that.
#[derive(Default)]
struct PlainFieldVisitor {
    fields: Vec<(String, String)>,
}

impl PlainFieldVisitor {
    fn set(&mut self, field: &Field, value: String) {
        match self.fields.iter_mut().find(|(name, _)| name == field.name()) {
            Some(existing) => existing.1 = value,
            None => self.fields.push((field.name().to_owned(), value)),
        }
    }

    fn into_string(self) -> String {
        self.fields
            .into_iter()
            .map(|(name, value)| format!("{name}={value}"))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

impl Visit for PlainFieldVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        self.set(field, format!("{value:?}"));
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        self.set(field, value.to_owned());
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        self.set(field, value.to_string());
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.set(field, value.to_string());
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.set(field, value.to_string());
    }
}

impl<S> Layer<S> for SlowSpanLayer
where
    S: Subscriber + for<'lookup> LookupSpan<'lookup>,
{
    fn on_new_span(
        &self,
        attrs: &span::Attributes<'_>,
        id: &span::Id,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        // Ignore tokio's internal runtime spans (enabled by `tokio_unstable` +
        // the `tracing` feature) — they fire constantly and drown out
        // application-level slow spans.
        let target = attrs.metadata().target();
        if target == "runtime" || target.starts_with("runtime.") || target.starts_with("tokio") {
            return;
        }

        if LONG_LIVED_SPAN_NAMES.contains(&attrs.metadata().name()) {
            return;
        }

        let mut visitor = PlainFieldVisitor::default();
        attrs.record(&mut visitor);
        self.fields.lock().unwrap().insert(id.clone(), visitor.fields);

        self.start_times
            .lock()
            .unwrap()
            .insert(id.clone(), Instant::now());
    }

    fn on_record(
        &self,
        id: &span::Id,
        values: &span::Record<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        // Only spans we're actually tracking (i.e. not one of the ignored
        // ones above) have an entry here.
        if let Some(existing) = self.fields.lock().unwrap().get_mut(id) {
            let mut visitor = PlainFieldVisitor {
                fields: std::mem::take(existing),
            };
            values.record(&mut visitor);
            *existing = visitor.fields;
        }
    }

    fn on_close(&self, id: span::Id, ctx: tracing_subscriber::layer::Context<'_, S>) {
        let fields = self.fields.lock().unwrap().remove(&id);

        let Some(started) = self.start_times.lock().unwrap().remove(&id) else {
            return;
        };

        let elapsed = started.elapsed();
        if elapsed < self.threshold {
            return;
        }

        let name = ctx
            .span(&id)
            .map(|span| span.metadata().name().to_owned())
            .unwrap_or_else(|| String::from("unknown"));
        let fields = fields
            .map(|fields| PlainFieldVisitor { fields }.into_string())
            .unwrap_or_default();

        let message = format!(
            "slow span '{}' took {:?}{}",
            name,
            elapsed,
            if fields.is_empty() {
                String::new()
            } else {
                format!(" ({})", fields)
            }
        );

        tracing::warn!(target: "reoserv::slow_span", "{}", message);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        io,
        sync::{Arc, Mutex},
    };

    #[derive(Clone)]
    struct TestWriter(Arc<Mutex<Vec<u8>>>);

    impl io::Write for TestWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    fn capture_with(
        threshold: Duration,
    ) -> (
        Arc<Mutex<Vec<u8>>>,
        impl tracing::Subscriber + Send + Sync + 'static,
    ) {
        capture_with_ansi(threshold, false)
    }

    fn capture_with_ansi(
        threshold: Duration,
        ansi: bool,
    ) -> (
        Arc<Mutex<Vec<u8>>>,
        impl tracing::Subscriber + Send + Sync + 'static,
    ) {
        let buf = Arc::new(Mutex::new(Vec::new()));
        let writer = TestWriter(buf.clone());

        let subscriber = tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer()
                    .with_writer(move || writer.clone())
                    .with_target(false)
                    .with_ansi(ansi),
            )
            .with(SlowSpanLayer::new(threshold));

        (buf, subscriber)
    }

    #[test]
    fn slow_span_emits_warning_when_over_threshold() {
        let (buf, subscriber) = capture_with(Duration::from_millis(0));

        tracing::subscriber::with_default(subscriber, || {
            let span = tracing::info_span!("test_span");
            let _enter = span.enter();
            std::thread::sleep(Duration::from_millis(5));
        });

        let output = String::from_utf8(buf.lock().unwrap().clone()).unwrap();
        assert!(
            output.contains("slow span"),
            "expected slow-span warning, got: {output}"
        );
    }

    #[test]
    fn slow_span_silent_when_under_threshold() {
        let (buf, subscriber) = capture_with(Duration::from_secs(3600));

        tracing::subscriber::with_default(subscriber, || {
            let span = tracing::info_span!("test_span");
            let _enter = span.enter();
        });

        let output = String::from_utf8(buf.lock().unwrap().clone()).unwrap();
        assert!(
            !output.contains("slow span"),
            "unexpected slow-span warning: {output}"
        );
    }

    #[test]
    fn ignores_tokio_runtime_spans() {
        let (buf, subscriber) = capture_with(Duration::from_millis(0));

        tracing::subscriber::with_default(subscriber, || {
            let span = tracing::info_span!(target: "runtime", "resource.async_op");
            let _enter = span.enter();
            std::thread::sleep(Duration::from_millis(5));
        });

        let output = String::from_utf8(buf.lock().unwrap().clone()).unwrap();
        assert!(
            !output.contains("slow span"),
            "tokio runtime spans should be ignored, got: {output}"
        );
    }

    #[test]
    fn ignores_actor_task_root_spans() {
        let (buf, subscriber) = capture_with(Duration::from_millis(0));

        tracing::subscriber::with_default(subscriber, || {
            for span in [
                tracing::info_span!("world"),
                tracing::info_span!("map"),
                tracing::info_span!("db"),
            ] {
                let _enter = span.enter();
                std::thread::sleep(Duration::from_millis(5));
            }
        });

        let output = String::from_utf8(buf.lock().unwrap().clone()).unwrap();
        assert!(
            !output.contains("slow span"),
            "actor task-root spans should be ignored, got: {output}"
        );
    }

    #[test]
    fn slow_span_fields_are_plain_text_even_with_ansi_enabled() {
        let (buf, subscriber) = capture_with_ansi(Duration::from_millis(0), true);

        tracing::subscriber::with_default(subscriber, || {
            let span = tracing::info_span!("packet", family = "Login", action = "Request");
            let _enter = span.enter();
            std::thread::sleep(Duration::from_millis(5));
        });

        let output = String::from_utf8(buf.lock().unwrap().clone()).unwrap();
        // The fmt layer still colors its own timestamp/level, but the
        // `(field=value ...)` segment we build ourselves must be plain text —
        // that's the part that used to leak literal escape codes.
        assert!(
            output.contains("(family=Login action=Request)"),
            "expected plain-text fields with no ANSI escapes, got: {output:?}"
        );
    }

    #[test]
    fn slow_span_fields_reflect_late_record_calls() {
        let (buf, subscriber) = capture_with(Duration::from_millis(0));

        tracing::subscriber::with_default(subscriber, || {
            let span = tracing::info_span!("player_session", character_name = tracing::field::Empty);
            let _enter = span.enter();
            span.record("character_name", "Bob");
            std::thread::sleep(Duration::from_millis(5));
        });

        let output = String::from_utf8(buf.lock().unwrap().clone()).unwrap();
        assert!(
            output.contains("character_name=Bob"),
            "expected recorded field to appear in slow-span message, got: {output}"
        );
    }
}
