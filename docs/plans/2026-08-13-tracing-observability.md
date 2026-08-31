# Tracing / Observability Implementation Plan

> **Branch:** `tracing/observability` (off `upstream/master`)
> **Repo:** `reoserv` (Rust, edition 2024)

**Goal:** Turn reoserv's tracing from "log macros with timestamps" into real
observability — named spans, per-connection character context, and tooling to
spot hot paths and slow execution.

**Architecture:** A single layered `tracing_subscriber` registry (fmt + console +
a custom slow-span layer). A per-connection `player_session` span carries
character identity, a per-packet span carries `family`/`action`, and a
`SlowSpanLayer` emits warnings for any span that overruns a configurable
threshold.

**Tech stack:** `tracing`, `tracing-subscriber` (env-filter, chrono, fmt),
`console-subscriber` (tokio-console), hand-rolled `SlowSpanLayer`.

---

## Current state assessment

What's already there (from `Cargo.toml` + `src/main.rs` + grep across `src/`):

- `tracing = "0.1"` and
  `tracing-subscriber = { version = "0.3", features = ["env-filter", "chrono", "fmt"] }`
- `console-subscriber = { version = "0.5", optional = true }` behind a `console` feature
- `tokio` has `features = ["full", "tracing"]`
- ~60 files call `tracing::info!` / `warn!` / `error!` / `trace!`

What's missing — this is the whole point of the work:

1. **Zero spans.** No `#[instrument]`, no `tracing::span!`, no `info_span!`
   anywhere. Every macro is a bare log line with no hierarchy, no timing, no
   structured fields.
2. **No structured fields.** Logs interpolate values into strings; nothing is
   queryable (e.g. `player_id=123`) via the fmt layer's field formatter.
3. **No character context.** Nothing ties a log line back to `character_id` /
   `character_name` / `admin_level`.
4. **Subscriber setup is broken for `console`.** `main.rs` calls
   `console_subscriber::init()` (feature-gated) *and then*
   `tracing_subscriber::fmt().init()` unconditionally. Both try to set the
   global subscriber — `fmt().init()` panics ("a global default trace
   dispatcher has already been set") when built with `--features console`.
5. **`console` feature can't work anyway.** `console-subscriber` needs tokio
   built with `--cfg tokio_unstable`; there's no `.cargo/config.toml` setting
   it, so the tokio task-tracing events are never emitted.

### Key insertion points (verified)

| Concern | File | Notes |
|---|---|---|
| Subscriber init | `src/main.rs:132-144` | replace two `.init()` calls with one registry |
| Per-connection task | `src/player/player_handle.rs:276` `run_player` | root span lives here |
| Player identity | `src/player/player.rs:14` `Player` | has `id`, `ip`, `character`, `character_id`, `character_name` |
| Character fields | `src/character.rs:43` `Character` | `id`, `account_id`, `name`, `admin_level: AdminLevel` |
| Character assigned | `src/player/player/account/select_character.rs:69-112` | `character_id`/`name` set at 69-70, `character = Some(...)` at 112 |
| Packet dispatch | `src/player/handle_packet.rs:12` `handle_packet` | `family`/`action` parsed by ~line 22 |
| Other task roots | `run_map` (`src/map/map_handle.rs:41`), `run_world` (`src/world/world_handle.rs:21`), `run_db` (`src/db/db_handle.rs:14`) | name them for tokio-console |

---

## Proposed architecture

### 1. Layered subscriber (one registry, three layers)

```rust
// src/observability.rs
use tracing_subscriber::{EnvFilter, fmt::{self, time::ChronoLocal}, prelude::*};

pub fn init_tracing() {
    if std::env::var("RUST_LOG").is_err() {
        unsafe { std::env::set_var("RUST_LOG", "info") }
    }

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    // EnvFilter applies ONLY to the fmt layer, so the console layer still
    // receives tokio's internal trace events (it does its own filtering).
    let fmt_layer = fmt::layer()
        .with_timer(ChronoLocal::new(String::from("%Y-%m-%d %I:%M:%S%.3f %p")))
        .with_target(true)
        .with_filter(env_filter);

    let registry = tracing_subscriber::registry()
        .with(fmt_layer)
        .with(SlowSpanLayer::new_from_env());

    #[cfg(feature = "console")]
    let registry = registry.with(console_subscriber::spawn());

    registry.init();
}
```

`main.rs` then just calls `crate::observability::init_tracing();` at the top of
`main()`.

**Why `with_filter` on the fmt layer (not a global filter):** a global
`EnvFilter::new("info")` would swallow tokio's `runtime.spawn`/`task` trace
events before they reach the console layer, making tokio-console show nothing.
Filtering per-layer is the documented correct pattern for console-subscriber.

### 2. Per-connection `player_session` span (character context)

Add a `span: tracing::Span` to `Player`:

```rust
// src/player/player.rs
pub struct Player {
    // ...existing fields...
    pub span: tracing::Span,
}
```

Create it in `Player::new` and instrument the connection loop with it:

```rust
// src/player/player_handle.rs, run_player
async fn run_player(mut player: Player) {
    let span = player.span.clone();
    async move {
        // ...existing select!/loop body...
    }
    .instrument(span)
    .await;
}
```

Record character identity onto the span once known — add a helper:

```rust
// src/player/player.rs, impl Player
pub fn record_character(&self, character: &Character) {
    self.span.record("character_id", character.id);
    self.span.record("character_name", character.name.as_str());
    self.span.record("admin_level", i32::from(character.admin_level));
}
```

Call it in `select_character.rs` right after `self.character = Some(character);`
(line 112). Every log emitted from the connection task thereafter inherits the
character fields.

### 3. Per-packet span

In `handle_packet`, after `family`/`action` are parsed, wrap the dispatch:

```rust
// src/player/handle_packet.rs
let span = tracing::info_span!("packet", family = ?family, action = ?action, player_id = self.id);
let _enter = span.enter();
match family { /* ...existing dispatch... */ }
```

This gives per-packet timing and lets `SlowSpanLayer` (and `RUST_LOG=debug`)
report exactly which packet family/action is slow, tied to `player_id` via the
parent `player_session` span.

### 4. `SlowSpanLayer` (slow-execution detection)

Hand-rolled layer (no new runtime dep) that logs any span exceeding a threshold:

```rust
// src/observability.rs
pub struct SlowSpanLayer {
    threshold: Duration,
    start_times: Mutex<HashMap<span::Id, Instant>>,
}
```

- `on_new_span` records `Instant::now()`.
- `on_close` computes elapsed; if `>= threshold`, emits
  `tracing::warn!(target: "reoserv::slow_span", elapsed_ms, "slow span")`,
  pulling span name + formatted fields out of `FormattedFields` from span
  extensions (available because the fmt layer is in the same registry).
- Threshold configurable via env (`SLOW_SPAN_MS`, default e.g. `100`).

This is the single highest-value piece for "spot slow execution" — it surfaces
*which* packet/handler is slow and by how much, with character context attached,
with zero profiling rigmarole.

### 5. Name the long-lived tasks (hot-path visibility in tokio-console)

Add `#[instrument(name = "...", skip(...))]` to the spawned task roots so
tokio-console's task list and the fmt layer show real names instead of
opaque futures:

- `run_player` → `"player"` (already gets the `player_session` span; just ensure the task is named)
- `run_map` → `"map"` (`src/map/map_handle.rs:41`)
- `run_world` → `"world"` (`src/world/world_handle.rs:21`)
- `run_db` → `"db"` (`src/db/db_handle.rs:14`)

---

## Phased tasks

### Phase A — Subscriber foundation

**Task A1.** Add `.cargo/config.toml`:
```toml
[build]
rustflags = ["--cfg", "tokio_unstable"]
```

**Task A2.** Create `src/observability.rs` with `init_tracing()` (layered
registry: fmt + slow-span stub) and a minimal `SlowSpanLayer` (threshold only,
no field formatting yet). Add `mod observability;` to `main.rs`, replace the
`console_subscriber::init()` + `fmt().init()` block with
`crate::observability::init_tracing();`.

- Verify: `cargo run` compiles, logs look identical to before.
- Verify: `cargo build --features console` compiles (no double-init panic).

**Task A3.** Move the console layer into the registry (`#[cfg(feature = "console")]`)
and add `SLOW_SPAN_MS` env parsing to `SlowSpanLayer`.

### Phase B — Character context

**Task B1.** Add `span: tracing::Span` to `Player`, init in `Player::new`
(`info_span!("player_session", player_id = id, ip = %ip)`).

**Task B2.** Instrument `run_player` with the span (extract loop body into an
`async move` block, `.instrument(span)`).

**Task B3.** Add `Player::record_character`, call from `select_character.rs:112`.

- Verify: with `RUST_LOG=reoserv=debug`, a login produces log lines whose span
  context shows `player_session{player_id=N ip="..."}` and, after character
  select, `character_id`/`character_name`/`admin_level`.

### Phase C — Packet span + slow detection

**Task C1.** Wrap the `handle_packet` dispatch in an `info_span!("packet", ...)`.

**Task C2.** Finish `SlowSpanLayer` field formatting via `FormattedFields`, emit
the `reoserv::slow_span` warning.

- Verify: temporarily `SLOW_SPAN_MS=0`, exercise a packet, observe
  `slow span: packet{family=... action=...}` warnings with player context.

### Phase D — Task naming + targeted rollout

**Task D1.** `#[instrument]` the `run_map`/`run_world`/`run_db` roots.

**Task D2.** Instrument high-frequency/hot paths selectively (NOT all 60 files):
map event loops (`act_npcs`, `spawn_npcs`, `timed_spikes`), `tick`, and the
`db::query` path. Use `#[instrument(skip(...), level = "debug")]`.

**Task D3.** Clean up the raw `tracing::trace!("Recv: {:?}", &packet[4..])`
(leaks raw bytes) — gate behind a `packet_bytes` field/flag or remove.

---

## Testing strategy

- **Span structure** — dev-dependency `tracing-test = "0.2"`:
  - assert `player_session` span carries `player_id`/`ip`;
  - assert `packet` span carries `family`/`action`;
  - assert `record_character` sets `character_id`/`character_name`/`admin_level`.
- **SlowSpanLayer** — `tracing::subscriber::with_default` with a capture layer:
  - threshold 0 + a `sleep(1ms)` span → warning emitted;
  - huge threshold → no warning.
- **Smoke** — `cargo build --features console` + brief `cargo run` to confirm no
  panic and `tokio-console` shows named tasks.

---

## Risks & open questions

1. **`tokio_unstable` affects every build.** Adding it to `.cargo/config.toml`
   changes tokio's compiled code paths globally (console features baked in).
   Low risk, but worth a note in the PR. Alternative: only set `RUSTFLAGS` when
   building `--features console` (more friction).
2. **Per-packet span overhead.** One `info_span!` + one HashMap insert/remove
   per packet in `SlowSpanLayer` — nanoseconds, negligible at EO's packet rates.
   The fmt layer won't *print* packet spans unless `RUST_LOG` enables debug.
3. **Character info is late.** `character_*` fields are absent until character
   select completes; early packets (init/login handshake) log with only
   `player_id`/`ip`. Expected; note in PR.
4. **`admin_level` mapping** — `AdminLevel` is `Copy` + `From<AdminLevel> for i32`
   exists (used elsewhere); record as `i32` to keep fmt output clean.
5. **Field formatting in `SlowSpanLayer`** requires the fmt layer present in the
   registry (it is). If the fmt layer is ever made conditional, `FormattedFields`
   extraction must be guarded.

## Non-goals (YAGNI)

- OpenTelemetry / Prometheus metrics export — useful later, but spans +
  slow-span layer + tokio-console already answer "hot paths / slow execution".
- `tracing-timing` histograms — hand-rolled `SlowSpanLayer` covers the need
  with zero extra deps.
- Instrumenting all 60 files — the packet span gives per-packet granularity;
  per-handler spans are added only where profiling shows hotspots.
