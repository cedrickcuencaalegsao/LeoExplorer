use std::sync::Arc;

use shared::{CacheEvent, Reporter};

fn main() -> iced::Result {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info,cache-tree=trace"),
    )
    .init();

    // First run (no cache file yet): the GUI starts on the setup screen and
    // the settings + cache are built on a background thread.
    if bootstrap::needs_setup() {
        log::info!("No cache found, starting first-run setup.");
        return leoexplorer_gui::run(Some(bootstrap::spawn()));
    } 

    // Normal run: same behaviour as before, straight into the main window.
    match system::init_settings() {
        Ok(()) => {
            log::info!("Settings OK. Starting cache.");
            let report: Reporter = Arc::new(|event: CacheEvent| {
                log::trace!(target: "cache-tree", "{event:?}");
            });
            cache::init(report);
        }
        Err(e) => log::error!("Settings init failed, cache not started: {e}"),
    }

    leoexplorer_gui::run(None)
}