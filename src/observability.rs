use std::{
    collections::HashMap,
    sync::Mutex,
    time::{Duration, Instant},
};

use tracing::{Subscriber, span};
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
pub struct SlowSpanLayer {
    threshold: Duration,
    start_times: Mutex<HashMap<span::Id, Instant>>,
}

impl SlowSpanLayer {
    pub fn new(threshold: Duration) -> Self {
        Self {
            threshold,
            start_times: Mutex::new(HashMap::new()),
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

impl<S> Layer<S> for SlowSpanLayer
where
    S: Subscriber + for<'lookup> LookupSpan<'lookup>,
{
    fn on_new_span(
        &self,
        _attrs: &span::Attributes<'_>,
        id: &span::Id,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        self.start_times
            .lock()
            .unwrap()
            .insert(id.clone(), Instant::now());
    }

    fn on_close(&self, id: span::Id, ctx: tracing_subscriber::layer::Context<'_, S>) {
        let Some(started) = self.start_times.lock().unwrap().remove(&id) else {
            return;
        };

        let elapsed = started.elapsed();
        if elapsed < self.threshold {
            return;
        }

        let (name, fields) = match ctx.span(&id) {
            Some(span) => {
                let fields = span
                    .extensions()
                    .get::<fmt::FormattedFields<fmt::format::DefaultFields>>()
                    .map(|formatted| formatted.fields.clone())
                    .unwrap_or_default();
                (span.metadata().name().to_owned(), fields)
            }
            None => (String::from("unknown"), String::new()),
        };

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
