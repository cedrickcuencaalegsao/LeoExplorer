//! First-run setup: decides whether it is needed and runs it, reporting
//! progress through a channel the GUI can listen to.

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use shared::BootEvent;
use shared::CacheEvent;
use shared::Reporter;
use shared::Step;

pub type BootReceiver = UnboundedReceiver<BootEvent>;

/// True when the cache file is missing (first run, or it was deleted).
pub fn needs_setup() -> bool {
    !base_path::find_cache()
}

/// Runs settings + cache setup and returns the channel to follow it.
pub fn spawn() -> BootReceiver {
    let (tx, rx) = unbounded();

    // Settings run on a short-lived thread so the window opens immediately.
    // `cache::init` spawns its own thread and returns at once, so completion
    // is detected from `CacheEvent::Ready` in the reporter, not from its return.
    thread::Builder::new()
        .name("bootstrap".into())
        .spawn(move || {
            let send = |event: BootEvent| {
                let _ = tx.unbounded_send(event);
            };

            send(BootEvent::Step(Step::Settings));
            if let Err(e) = system::init_settings() {
                log::error!("Settings init failed, cache not started: {e}");
                send(BootEvent::Failed(format!("Could not set up settings: {e}")));
                return;
            }
            log::info!("Settings OK. Starting cache.");

            send(BootEvent::Step(Step::Cache));
            cache::init(reporter(tx.clone()));
        })
        .expect("failed to spawn bootstrap thread");

    rx
}

/// Maps a cache event to a 0.0..=1.0 progress value.
///
/// TODO: needs the real `CacheEvent` variants that carry progress.
fn percent(_event: &CacheEvent) -> Option<f32> {
    None
}

/// Human-readable text for an event (falls back to Debug for other variants).
fn describe(event: &CacheEvent) -> String {
    match event {
        CacheEvent::Started(msg)
        | CacheEvent::Creating(msg)
        | CacheEvent::Progress(msg)
        | CacheEvent::Completed(msg)
        | CacheEvent::ExtensionsReady(msg)
        | CacheEvent::Error(msg)
        | CacheEvent::Ready(msg) => msg.to_string(),
        #[allow(unreachable_patterns)]
        other => format!("{other:?}"),
    }
}

fn reporter(tx: UnboundedSender<BootEvent>) -> Reporter {
    let last = Mutex::new(None::<Instant>);

    Arc::new(move |event: CacheEvent| {
        log::trace!(target: "cache-tree", "{event:?}");

        // Terminal events are never throttled.
        match &event {
            // `Ready`: the cache already existed.
            // `ExtensionsReady`: last event of a fresh build (cache written,
            // extensions saved to settings), so the main window can open.
            CacheEvent::Ready(_) | CacheEvent::ExtensionsReady(_) => {
                let _ = tx.unbounded_send(BootEvent::Ready);
                return;
            }
            CacheEvent::Error(msg) => {
                let _ = tx.unbounded_send(BootEvent::Failed(msg.to_string()));
                return;
            }
            _ => {}
        }

        // Cache events can arrive thousands of times per second; forward at
        // most one every 80 ms.
        let mut last = last.lock().unwrap();
        let now = Instant::now();
        if last.map_or(true, |t| now.duration_since(t) >= Duration::from_millis(80)) {
            *last = Some(now);

            if let Some(p) = percent(&event) {
                let _ = tx.unbounded_send(BootEvent::Progress(p));
            }

            let mut label = describe(&event);
            label.truncate(120);
            let _ = tx.unbounded_send(BootEvent::Detail(label));
        }
    })
}
