use std::sync::{Arc, mpsc};
use shared::{CacheEvent, Reporter};

fn main() -> iced::Result {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info,cache-tree=trace")).init();

    let (_tx, _rx) = mpsc::channel::<CacheEvent>();

    let report: Reporter = Arc::new(move |event| {
        let _ = _tx.send(event);
    });
    match system::init_settings()  {
        Ok(()) =>{
            log::info!("Settings OK. Starting cache.");
            cache::init(report);
        }
        Err(e) => {
            log::error!("Settings init failed, cache not started: {e}")
        }
    }

    // hand `rx` to the GUI, e.g. via a static/OnceLock or a custom run function
    leoexplorer_gui::run()
}